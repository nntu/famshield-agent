package api

import (
	"encoding/json"
	"net/http"

	"childprotect-server/pkg/model"
	"childprotect-server/pkg/store"
)

type Server struct {
	store *store.MemoryStore
	mux   *http.ServeMux
}

func NewServer(s *store.MemoryStore) *Server {
	srv := &Server{
		store: s,
		mux:   http.NewServeMux(),
	}
	srv.registerRoutes()
	return srv
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

