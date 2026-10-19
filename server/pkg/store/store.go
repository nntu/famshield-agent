package store

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"sync"
	"time"

	"famshield-server/pkg/model"
)

type MemoryStore struct {
	mu              sync.RWMutex
	devices         map[string]*model.Device
	enrollmentCodes map[string]*model.EnrollmentCode
	commands        map[string]*model.DeviceCommand
	events          map[string]*model.IngestedEvent
	evidence        map[string]*model.VisualEvidenceMetadata
	latestRelease   *model.ReleaseManifest
	users           map[string]*model.User
	usersByEmail    map[string]*model.User
	usersByToken    map[string]*model.User
	usersByCode     map[string]*model.User
	usersByChatID   map[int64]*model.User
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{
		devices:         make(map[string]*model.Device),
		enrollmentCodes: make(map[string]*model.EnrollmentCode),
		commands:        make(map[string]*model.DeviceCommand),
		events:          make(map[string]*model.IngestedEvent),
		evidence:        make(map[string]*model.VisualEvidenceMetadata),
		users:           make(map[string]*model.User),
		usersByEmail:    make(map[string]*model.User),
		usersByToken:    make(map[string]*model.User),
		usersByCode:     make(map[string]*model.User),
		usersByChatID:   make(map[int64]*model.User),
		latestRelease: &model.ReleaseManifest{
			ManifestVersion:     1,
			Version:             "0.1.1",
			Channel:             "stable",
			MinSupportedVersion: "0.1.0",
			PublishedAt:         time.Now().UTC(),
			ServiceSHA256:       "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			ServiceBinaryURL:    "https://dl.famshield.cloud/releases/v0.1.1/famshield-service.exe",
			SignatureHex:        "3045022100abc...",
		},
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

// GetLatestRelease returns the current active release manifest.
func (s *MemoryStore) GetLatestRelease() *model.ReleaseManifest {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.latestRelease
}

// PublishRelease updates the active release manifest.
func (s *MemoryStore) PublishRelease(release *model.ReleaseManifest) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.latestRelease = release
}

// RegisterUser creates a new user in pending_activation status with an activation token and OTP code.
func (s *MemoryStore) RegisterUser(email, passwordHash, telegramUsername string) (*model.User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.usersByEmail[email]; exists {
		return nil, fmt.Errorf("user with email %s already exists", email)
	}

	// Generate activation token (random 16 hex chars)
	tokenBytes := make([]byte, 8)
	if _, err := rand.Read(tokenBytes); err != nil {
		return nil, err
	}
	token := fmt.Sprintf("act_%x", tokenBytes)

	// Generate 6-digit OTP code
	n, err := rand.Int(rand.Reader, big.NewInt(900000))
	if err != nil {
		return nil, err
	}
	code := fmt.Sprintf("%06d", n.Int64()+100000)

	userID := fmt.Sprintf("usr-%x", tokenBytes[:4])
	tenantID := fmt.Sprintf("tenant-%s", userID)

	now := time.Now().UTC()
	user := &model.User{
		ID:               userID,
		Email:            email,
		PasswordHash:     passwordHash,
		Status:           model.UserStatusPendingActivation,
		TenantID:         tenantID,
		TelegramUsername: telegramUsername,
		ActivationToken:  token,
		ActivationCode:   code,
		TokenExpiresAt:   now.Add(24 * time.Hour), // Valid for 24 hours
		CreatedAt:        now,
	}

	s.users[userID] = user
	s.usersByEmail[email] = user
	s.usersByToken[token] = user
	s.usersByCode[code] = user

	return user, nil
}

// ActivateUserByToken activates user via Telegram deep link /start <token>.
func (s *MemoryStore) ActivateUserByToken(token string, telegramChatID int64, telegramUsername string) (*model.User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	user, exists := s.usersByToken[token]
	if !exists {
		return nil, fmt.Errorf("invalid activation token")
	}

	now := time.Now().UTC()
	if !user.IsTokenValid(now) {
		return nil, fmt.Errorf("activation token expired or already used")
	}

	user.Status = model.UserStatusActive
	user.ActivatedAt = &now
	user.TelegramChatID = telegramChatID
	if telegramUsername != "" {
		user.TelegramUsername = telegramUsername
	}

	if telegramChatID != 0 {
		s.usersByChatID[telegramChatID] = user
	}

	return user, nil
}

// ActivateUserByCode activates user via 6-digit OTP code.
func (s *MemoryStore) ActivateUserByCode(code string) (*model.User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	user, exists := s.usersByCode[code]
	if !exists {
		return nil, fmt.Errorf("invalid activation code")
	}

	now := time.Now().UTC()
	if !user.IsTokenValid(now) {
		return nil, fmt.Errorf("activation code expired or already used")
	}

	user.Status = model.UserStatusActive
	user.ActivatedAt = &now
	return user, nil
}

// GetUserByID retrieves a user by their user ID.
func (s *MemoryStore) GetUserByID(userID string) (*model.User, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	u, ok := s.users[userID]
	return u, ok
}

// GetUserByEmail retrieves a user by email.
func (s *MemoryStore) GetUserByEmail(email string) (*model.User, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	u, ok := s.usersByEmail[email]
	return u, ok
}

// GetUserByTelegramChatID retrieves a user by Telegram Chat ID.
func (s *MemoryStore) GetUserByTelegramChatID(chatID int64) (*model.User, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	u, ok := s.usersByChatID[chatID]
	return u, ok
}


