package model

import "time"

// Tenant represents an isolated customer organization.
type Tenant struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	CreatedAt time.Time `json:"created_at"`
}

// Device represents an enrolled Windows endpoint.
type Device struct {
	ID          string    `json:"id"`
	TenantID    string    `json:"tenant_id"`
	ChildID     *string   `json:"child_id,omitempty"`
	DeviceName  string    `json:"device_name"`
	OSVersion   string    `json:"os_version"`
	Revoked     bool      `json:"revoked"`
	LastSeenAt  time.Time `json:"last_seen_at"`
	CreatedAt   time.Time `json:"created_at"`
}

// EnrollmentCode represents a 15-minute, single-use 6-digit enrollment code.
type EnrollmentCode struct {
	Code      string    `json:"code"`
	TenantID  string    `json:"tenant_id"`
	ChildID   *string   `json:"child_id,omitempty"`
	Used      bool      `json:"used"`
	ExpiresAt time.Time `json:"expires_at"`
	CreatedAt time.Time `json:"created_at"`
}

func (e *EnrollmentCode) IsValid(now time.Time) bool {
	return !e.Used && now.Before(e.ExpiresAt)
}

// DeviceCommand represents an asynchronous action queued for the agent.
type DeviceCommand struct {
	CommandID string         `json:"command_id"`
	DeviceID  string         `json:"device_id"`
	Type      string         `json:"type"` // e.g. "sync_policy", "lock_session"
	Payload   map[string]any `json:"payload"`
	State     string         `json:"state"` // "pending", "delivered", "completed"
	ExpiresAt time.Time      `json:"expires_at"`
	CreatedAt time.Time      `json:"created_at"`
}

// IngestedEvent represents an event posted in a batch from the agent outbox.
type IngestedEvent struct {
	EventID    string         `json:"event_id"`
	DeviceID   string         `json:"device_id"`
	ChildID    *string        `json:"child_id,omitempty"`
	EventType  string         `json:"event_type"`
	OccurredAt time.Time      `json:"occurred_at"`
	Payload    map[string]any `json:"payload"`
}

// VisualEvidenceMetadata represents metadata for a privacy-blurred evidence frame.
type VisualEvidenceMetadata struct {
	EvidenceID string    `json:"evidence_id"`
	DeviceID   string    `json:"device_id"`
	EventID    string    `json:"event_id"`
	Trigger    string    `json:"trigger"`
	SizeBytes  int64     `json:"size_bytes"`
	ExpiresAt  time.Time `json:"expires_at"` // 48-Hour Retention
	CreatedAt  time.Time `json:"created_at"`
}
