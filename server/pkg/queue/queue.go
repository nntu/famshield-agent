package queue

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"

	"famshield-server/pkg/model"
)

const (
	StreamName     = "FAMSHIELD_EVENTS"
	StreamSubjects = "famshield.events.>"
	ConsumerName   = "event-processor-group"
)

// QueueManager coordinates publishing events to JetStream and consuming them via background workers.
type QueueManager struct {
	conn *nats.Conn
	js   jetstream.JetStream
}

func NewQueueManager(natsURL string) (*QueueManager, error) {
	if natsURL == "" {
		natsURL = nats.DefaultURL
	}

	nc, err := nats.Connect(natsURL,
		nats.Name("famshield-control-plane"),
		nats.Timeout(5*time.Second),
		nats.ReconnectWait(2*time.Second),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to NATS: %w", err)
	}

	js, err := jetstream.New(nc)
	if err != nil {
		nc.Close()
		return nil, fmt.Errorf("failed to create JetStream context: %w", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Ensure durable stream exists with 7-day retention
	_, err = js.CreateOrUpdateStream(ctx, jetstream.StreamConfig{
		Name:      StreamName,
		Subjects:  []string{StreamSubjects},
		Retention: jetstream.LimitsPolicy,
		MaxAge:    7 * 24 * time.Hour,
		Storage:   jetstream.FileStorage,
	})
	if err != nil {
		log.Printf("Warning: Could not create persistent stream (falling back to memory): %v", err)
		_, _ = js.CreateOrUpdateStream(ctx, jetstream.StreamConfig{
			Name:      StreamName,
			Subjects:  []string{StreamSubjects},
			Retention: jetstream.LimitsPolicy,
			MaxAge:    24 * time.Hour,
			Storage:   jetstream.MemoryStorage,
		})
	}

	return &QueueManager{
		conn: nc,
		js:   js,
	}, nil
}

// PublishEvent publishes an ingested event to the stream asynchronously.
func (qm *QueueManager) PublishEvent(ctx context.Context, event *model.IngestedEvent) error {
	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to serialize event: %w", err)
	}

	subject := fmt.Sprintf("famshield.events.%s", event.EventType)
	_, err = qm.js.Publish(ctx, subject, data)
	return err
}

// StartWorker starts a durable consumer worker pulling events from JetStream.
func (qm *QueueManager) StartWorker(ctx context.Context, handler func(event *model.IngestedEvent) error) error {
	stream, err := qm.js.Stream(ctx, StreamName)
	if err != nil {
		return fmt.Errorf("failed to bind to stream %s: %w", StreamName, err)
	}

	cons, err := stream.CreateOrUpdateConsumer(ctx, jetstream.ConsumerConfig{
		Durable:       ConsumerName,
		AckPolicy:     jetstream.AckExplicitPolicy,
		FilterSubject: StreamSubjects,
	})
	if err != nil {
		return fmt.Errorf("failed to create consumer: %w", err)
	}

	go func() {
		iter, err := cons.Messages()
		if err != nil {
			log.Printf("Consumer error: %v", err)
			return
		}
		for {
			select {
			case <-ctx.Done():
				return
			default:
				msg, err := iter.Next()
				if err != nil {
					continue
				}

				var evt model.IngestedEvent
				if err := json.Unmarshal(msg.Data(), &evt); err == nil {
					if err := handler(&evt); err == nil {
						_ = msg.Ack()
					}
				}
			}
		}
	}()

	return nil
}

func (qm *QueueManager) Close() {
	if qm.conn != nil {
		qm.conn.Close()
	}
}
