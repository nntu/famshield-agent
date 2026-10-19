package store

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"sync"
	"time"

	"childprotect-server/pkg/model"
)

type MemoryStore struct {
	mu              sync.RWMutex
	devices         map[string]*model.Device
	enrollmentCodes map[string]*model.EnrollmentCode
	commands        map[string]*model.DeviceCommand
	events          map[string]*model.IngestedEvent
	evidence        map[string]*model.VisualEvidenceMetadata
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{
		devices:         make(map[string]*model.Device),
		enrollmentCodes: make(map[string]*model.EnrollmentCode),
		commands:        make(map[string]*model.DeviceCommand),
		events:          make(map[string]*model.IngestedEvent),
		evidence:        make(map[string]*model.VisualEvidenceMetadata),
	}
}

// GenerateEnrollmentCode creates a 6-digit random code valid for 15 minutes.
func (s *MemoryStore) GenerateEnrollmentCode(tenantID string, childID *string) (*model.EnrollmentCode, error) {
	n, err := rand.Int(rand.Reader, big.NewInt(900000))
	if err != nil {
		return nil, err
	}
	codeStr := fmt.Sprintf("%06d", n.Int64()+100000)

	s.mu.Lock()
	defer s.mu.Unlock()

	code := &model.EnrollmentCode{
		Code:      codeStr,
		TenantID:  tenantID,
		ChildID:   childID,
		Used:      false,
		ExpiresAt: time.Now().UTC().Add(15 * time.Minute),
		CreatedAt: time.Now().UTC(),
	}
	s.enrollmentCodes[codeStr] = code
	return code, nil
}

// RedeemEnrollmentCode validates and marks the code as used, registering a new Device.
func (s *MemoryStore) RedeemEnrollmentCode(codeStr, deviceName, osVersion string) (*model.Device, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	code, exists := s.enrollmentCodes[codeStr]
	if !exists || !code.IsValid(time.Now().UTC()) {
		return nil, fmt.Errorf("invalid or expired enrollment code")
	}

	code.Used = true
	deviceID := fmt.Sprintf("dev-%s", codeStr)
	device := &model.Device{
		ID:         deviceID,
		TenantID:   code.TenantID,
		ChildID:    code.ChildID,
		DeviceName: deviceName,
		OSVersion:  osVersion,
		Revoked:    false,
		LastSeenAt: time.Now().UTC(),
		CreatedAt:  time.Now().UTC(),
	}
	s.devices[deviceID] = device
	return device, nil
}

// SaveEvent saves an ingested event idempotently.
func (s *MemoryStore) SaveEvent(event *model.IngestedEvent) bool {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.events[event.EventID]; exists {
		return false // duplicate
	}
	s.events[event.EventID] = event
	return true
}

// SaveEvidence stores evidence metadata with 48h expiration.
func (s *MemoryStore) SaveEvidence(evi *model.VisualEvidenceMetadata) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.evidence[evi.EvidenceID] = evi
}

// GetDevice retrieves a device by ID.
func (s *MemoryStore) GetDevice(deviceID string) (*model.Device, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	d, ok := s.devices[deviceID]
	return d, ok
}
