package api_test

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"famshield-server/pkg/api"
	"famshield-server/pkg/model"
	"famshield-server/pkg/store"
)

func TestDeviceEnrollmentFlow(t *testing.T) {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	// 1. Generate 6-digit code
	code, err := st.GenerateEnrollmentCode("tenant-abc", nil)
	if err != nil {
		t.Fatalf("Failed to generate code: %v", err)
	}

	if len(code.Code) != 6 {
		t.Fatalf("Expected 6-digit code, got %s", code.Code)
	}

	// 2. Submit valid enrollment
	enrollReq := api.EnrollRequest{
		EnrollmentCode: code.Code,
		DeviceName:     "Child-Gaming-Laptop",
		OSVersion:      "Windows 11 Home",
	}
	body, _ := json.Marshal(enrollReq)

	req := httptest.NewRequest(http.MethodPost, "/v1/device/enroll", bytes.NewReader(body))
	w := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("Expected 200 OK, got %d: %s", w.Code, w.Body.String())
	}

	var resp api.EnrollResponse
	if err := json.NewDecoder(w.Body).Decode(&resp); err != nil {
		t.Fatalf("Failed to decode response: %v", err)
	}

	if resp.DeviceID != "dev-"+code.Code {
		t.Fatalf("Unexpected device ID: %s", resp.DeviceID)
	}

	// 3. Reject reuse of enrollment code
	req2 := httptest.NewRequest(http.MethodPost, "/v1/device/enroll", bytes.NewReader(body))
	w2 := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w2, req2)

	if w2.Code != http.StatusUnauthorized {
		t.Fatalf("Expected 401 Unauthorized for reused code, got %d", w2.Code)
	}
}

func TestBatchEventsIngestion(t *testing.T) {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	batchReq := api.BatchEventsRequest{
		BatchID: "batch-101",
		Events: []model.IngestedEvent{
			{
				EventID:   "evt-1",
				DeviceID:  "dev-1",
				EventType: "app.blocked",
			},
			{
				EventID:   "evt-2",
				DeviceID:  "dev-1",
				EventType: "domain.blocked",
			},
		},
	}
	body, _ := json.Marshal(batchReq)

	req := httptest.NewRequest(http.MethodPost, "/v1/device/events:batch", bytes.NewReader(body))
	w := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("Expected 200 OK, got %d", w.Code)
	}

	var resp api.BatchEventsResponse
	json.NewDecoder(w.Body).Decode(&resp)

	if len(resp.Accepted) != 2 {
		t.Fatalf("Expected 2 accepted events, got %d", len(resp.Accepted))
	}

	// Post same batch again -> expect duplicate detection
	reqDuplicate := httptest.NewRequest(http.MethodPost, "/v1/device/events:batch", bytes.NewReader(body))
	wDuplicate := httptest.NewRecorder()
	srv.Handler().ServeHTTP(wDuplicate, reqDuplicate)

	var respDup api.BatchEventsResponse
	json.NewDecoder(wDuplicate.Body).Decode(&respDup)
	if len(respDup.Duplicate) != 2 {
		t.Fatalf("Expected 2 duplicate events, got %d", len(respDup.Duplicate))
	}
}

func TestHeartbeatAutoUpdateDetection(t *testing.T) {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	// Heartbeat from agent running older version 0.1.0 (latest is 0.1.1)
	hbReq := api.HeartbeatRequest{
		DeviceID:      "dev-001",
		AgentVersion:  "0.1.0",
		PolicyVersion: 1,
		HealthStatus:  "healthy",
	}
	body, _ := json.Marshal(hbReq)

	req := httptest.NewRequest(http.MethodPost, "/v1/device/heartbeat", bytes.NewReader(body))
	w := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("Expected 200 OK, got %d", w.Code)
	}

	var resp api.HeartbeatResponse
	if err := json.NewDecoder(w.Body).Decode(&resp); err != nil {
		t.Fatalf("Failed to decode response: %v", err)
	}

	if resp.AvailableRelease == nil {
		t.Fatalf("Expected available_release update metadata, got nil")
	}

	if resp.AvailableRelease.Version != "0.1.1" {
		t.Fatalf("Expected version 0.1.1, got %s", resp.AvailableRelease.Version)
	}
}

