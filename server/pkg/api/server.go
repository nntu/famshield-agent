package api

import (
	"encoding/json"
	"net/http"
	"os"

	"famshield-server/pkg/model"
	"famshield-server/pkg/queue"
	"famshield-server/pkg/store"
	"famshield-server/pkg/telegram"
)

type Server struct {
	store    *store.MemoryStore
	queue    *queue.QueueManager
	telegram *telegram.Service
	mux      *http.ServeMux
}

func NewServer(s *store.MemoryStore) *Server {
	srv := &Server{
		store:    s,
		telegram: telegram.NewService(os.Getenv("TELEGRAM_BOT_TOKEN"), os.Getenv("TELEGRAM_BOT_USERNAME"), s),
		mux:      http.NewServeMux(),
	}
	srv.registerRoutes()
	return srv
}

func (s *Server) SetQueue(q *queue.QueueManager) {
	s.queue = q
}

func (s *Server) SetTelegram(t *telegram.Service) {
	s.telegram = t
}

func (s *Server) Handler() http.Handler {
	return s.mux
}

func (s *Server) registerRoutes() {
	s.mux.HandleFunc("/v1/device/enroll", s.handleDeviceEnroll)
	s.mux.HandleFunc("/v1/device/events:batch", s.handleEventsBatch)
	s.mux.HandleFunc("/v1/device/heartbeat", s.handleHeartbeat)
	s.mux.HandleFunc("/v1/releases/latest", s.handleGetLatestRelease)
	s.mux.HandleFunc("/v1/admin/enrollment-codes", s.handleGenerateEnrollmentCode)
	s.mux.HandleFunc("/v1/auth/register", s.handleRegister)
	s.mux.HandleFunc("/v1/auth/activate", s.handleActivate)
	s.mux.HandleFunc("/v1/auth/status", s.handleAuthStatus)
	s.mux.HandleFunc("/v1/telegram/webhook", s.handleTelegramWebhook)
	s.mux.HandleFunc("/v1/download/agent", s.handleDownloadAgent)
	s.mux.HandleFunc("/healthz", s.handleHealthz)
}



func (s *Server) handleHealthz(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "healthy"})
}

type EnrollRequest struct {
	EnrollmentCode string `json:"enrollment_code"`
	DeviceName     string `json:"device_name"`
	OSVersion      string `json:"os_version"`
}

type EnrollResponse struct {
	DeviceID             string `json:"device_id"`
	TenantID             string `json:"tenant_id"`
	ClientCertificatePEM string `json:"client_certificate_pem"`
	CACertificatePEM     string `json:"ca_certificate_pem"`
}

func (s *Server) handleDeviceEnroll(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req EnrollRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	device, err := s.store.RedeemEnrollmentCode(req.EnrollmentCode, req.DeviceName, req.OSVersion)
	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnauthorized)
		json.NewEncoder(w).Encode(map[string]any{
			"error": map[string]any{
				"code":    "invalid_enrollment_code",
				"message": err.Error(),
			},
		})
		return
	}

	resp := EnrollResponse{
		DeviceID:             device.ID,
		TenantID:             device.TenantID,
		ClientCertificatePEM: "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----",
		CACertificatePEM:     "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----",
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}

type BatchEventsRequest struct {
	BatchID string                `json:"batch_id"`
	Events  []model.IngestedEvent `json:"events"`
}

type BatchEventsResponse struct {
	BatchID   string   `json:"batch_id"`
	Accepted  []string `json:"accepted"`
	Duplicate []string `json:"duplicate"`
}

func (s *Server) handleEventsBatch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req BatchEventsRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	accepted := make([]string, 0)
	duplicates := make([]string, 0)

	for _, event := range req.Events {
		e := event
		if s.store.SaveEvent(&e) {
			accepted = append(accepted, e.EventID)
			// Asynchronously publish to NATS JetStream
			if s.queue != nil {
				_ = s.queue.PublishEvent(r.Context(), &e)
			}
		} else {
			duplicates = append(duplicates, e.EventID)
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(BatchEventsResponse{
		BatchID:   req.BatchID,
		Accepted:  accepted,
		Duplicate: duplicates,
	})
}

type GenerateCodeRequest struct {
	TenantID string  `json:"tenant_id"`
	ChildID  *string `json:"child_id,omitempty"`
}

func (s *Server) handleGenerateEnrollmentCode(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req GenerateCodeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}
	if req.TenantID == "" {
		req.TenantID = "tenant-default"
	}

	code, err := s.store.GenerateEnrollmentCode(req.TenantID, req.ChildID)
	if err != nil {
		http.Error(w, "Internal server error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(code)
}

type HeartbeatRequest struct {
	DeviceID      string `json:"device_id"`
	AgentVersion  string `json:"agent_version"`
	PolicyVersion int    `json:"policy_version"`
	HealthStatus  string `json:"health_status"`
}

type HeartbeatResponse struct {
	ServerTime       string                 `json:"server_time"`
	DesiredPolicyVer int                    `json:"desired_policy_version"`
	AvailableRelease *model.ReleaseManifest `json:"available_release,omitempty"`
}

func (s *Server) handleHeartbeat(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req HeartbeatRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	rel := s.store.GetLatestRelease()
	var updateAvailable *model.ReleaseManifest
	if rel != nil && rel.Version != req.AgentVersion {
		updateAvailable = rel
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(HeartbeatResponse{
		ServerTime:       "2026-10-19T10:00:00Z",
		DesiredPolicyVer: 1,
		AvailableRelease: updateAvailable,
	})
}

func (s *Server) handleGetLatestRelease(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	rel := s.store.GetLatestRelease()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(rel)
}

type RegisterRequest struct {
	Email            string `json:"email"`
	Password         string `json:"password"`
	TelegramUsername string `json:"telegram_username,omitempty"`
}

type RegisterResponse struct {
	UserID          string `json:"user_id"`
	Email           string `json:"email"`
	Status          string `json:"status"`
	TelegramBotURL  string `json:"telegram_bot_url"`
	ActivationToken string `json:"activation_token"`
	ActivationCode  string `json:"activation_code"`
	ExpiresAt       string `json:"expires_at"`
}

func (s *Server) handleRegister(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req RegisterRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Email == "" || req.Password == "" {
		http.Error(w, "Email and password are required", http.StatusBadRequest)
		return
	}

	// Simple hash for password
	user, err := s.store.RegisterUser(req.Email, "hash_"+req.Password, req.TelegramUsername)
	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusConflict)
		json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	botURL := s.telegram.GetActivationDeepLink(user.ActivationToken)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(RegisterResponse{
		UserID:          user.ID,
		Email:           user.Email,
		Status:          user.Status,
		TelegramBotURL:  botURL,
		ActivationToken: user.ActivationToken,
		ActivationCode:  user.ActivationCode,
		ExpiresAt:       user.TokenExpiresAt.Format("2006-01-02T15:04:05Z07:00"),
	})
}

type ActivateRequest struct {
	Token string `json:"token,omitempty"`
	Code  string `json:"code,omitempty"`
}

func (s *Server) handleActivate(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req ActivateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	var user *model.User
	var err error
	if req.Token != "" {
		user, err = s.store.ActivateUserByToken(req.Token, 0, "")
	} else if req.Code != "" {
		user, err = s.store.ActivateUserByCode(req.Code)
	} else {
		http.Error(w, "Either token or code is required", http.StatusBadRequest)
		return
	}

	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]any{
		"status":  user.Status,
		"user_id": user.ID,
		"email":   user.Email,
		"message": "Tài khoản FamShield đã được kích hoạt thành công!",
	})
}

func (s *Server) handleAuthStatus(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	userID := r.URL.Query().Get("user_id")
	email := r.URL.Query().Get("email")

	var user *model.User
	var ok bool
	if userID != "" {
		user, ok = s.store.GetUserByID(userID)
	} else if email != "" {
		user, ok = s.store.GetUserByEmail(email)
	}

	if !ok {
		w.WriteHeader(http.StatusNotFound)
		json.NewEncoder(w).Encode(map[string]string{"error": "user not found"})
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]any{
		"user_id":      user.ID,
		"email":        user.Email,
		"status":       user.Status,
		"activated_at": user.ActivatedAt,
	})
}

func (s *Server) handleTelegramWebhook(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var update telegram.Update
	if err := json.NewDecoder(r.Body).Decode(&update); err != nil {
		http.Error(w, "Invalid payload", http.StatusBadRequest)
		return
	}

	_ = s.telegram.HandleUpdate(&update)

	w.WriteHeader(http.StatusOK)
}

func (s *Server) handleDownloadAgent(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// 1. Check if custom download URL is configured in environment
	customURL := os.Getenv("AGENT_DOWNLOAD_URL")
	if customURL != "" {
		http.Redirect(w, r, customURL, http.StatusFound)
		return
	}

	// 2. Check local built binary paths
	candidates := []string{
		"installer/Output/FamShieldSetup-0.1.0.exe",
		"../installer/Output/FamShieldSetup-0.1.0.exe",
		"target/release/famshield-service.exe",
		"../target/release/famshield-service.exe",
	}

	for _, path := range candidates {
		if fi, err := os.Stat(path); err == nil && !fi.IsDir() {
			w.Header().Set("Content-Disposition", "attachment; filename=\"FamShieldSetup.exe\"")
			http.ServeFile(w, r, path)
			return
		}
	}

	// 3. Fallback to GitHub Release download URL
	githubReleaseURL := "https://github.com/nntu/famshield/releases/latest/download/FamShieldSetup-0.1.0.exe"
	http.Redirect(w, r, githubReleaseURL, http.StatusFound)
}



