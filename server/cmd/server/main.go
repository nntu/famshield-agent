package main

import (
	"log"
	"net/http"

	"os"

	"famshield-server/pkg/api"
	"famshield-server/pkg/queue"
	"famshield-server/pkg/store"
)

func main() {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	// Connect to NATS JetStream if configured
	natsURL := os.Getenv("NATS_URL")
	if natsURL != "" {
		qm, err := queue.NewQueueManager(natsURL)
		if err != nil {
			log.Printf("Warning: Could not connect to NATS JetStream (%v); running in direct mode", err)
		} else {
			defer qm.Close()
			srv.SetQueue(qm)
			log.Printf("Connected to NATS JetStream at %s", natsURL)
		}
	}

	addr := ":8080"
	log.Printf("Starting FamShield Control Plane server on %s ...", addr)
	if err := http.ListenAndServe(addr, srv.Handler()); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}

