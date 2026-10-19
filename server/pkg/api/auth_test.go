package api

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"famshield-server/pkg/model"
	"famshield-server/pkg/store"
	"famshield-server/pkg/telegram"
)

func TestAuthRegisterAndActivate(t *testing.T) {
	st := store.NewMemoryStore()
	srv := NewServer(st)

	// 1. Register a new user
	regPayload := map[string]string{
		"email":             "mom@example.com",
		"password":          "supersecret123",
		"telegram_username": "mom_guardian",
	}
	body, _ := json.Marshal(regPayload)
	req := httptest.NewRequest(http.MethodPost, "/v1/auth/register", bytes.NewReader(body))
	w := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w, req)

	if w.Code != http.StatusCreated {
		t.Fatalf("Expected 201 Created, got %d: %s", w.Code, w.Body.String())
	}

	var regResp RegisterResponse
	if err := json.Unmarshal(w.Body.Bytes(), &regResp); err != nil {
		t.Fatalf("Failed to parse register response: %v", err)
	}

	if regResp.Status != model.UserStatusPendingActivation {
		t.Errorf("Expected status pending_activation, got %s", regResp.Status)
	}
	if regResp.TelegramBotURL == "" {
		t.Errorf("Expected non-empty TelegramBotURL")
	}

	// 2. Check Auth Status -> Pending
	statusReq := httptest.NewRequest(http.MethodGet, "/v1/auth/status?user_id="+regResp.UserID, nil)
	wStatus := httptest.NewRecorder()
	srv.Handler().ServeHTTP(wStatus, statusReq)

	if wStatus.Code != http.StatusOK {
		t.Fatalf("Expected 200 OK, got %d", wStatus.Code)
	}

	// 3. Simulate Telegram Webhook calling /start <token>
	webhookPayload := telegram.Update{
		UpdateID: 2001,
		Message: &telegram.Message{
			MessageID: 10,
			Chat:      &telegram.Chat{ID: 987654321, Type: "private"},
			Text:      "/start " + regResp.ActivationToken,
		},
	}
	webhookBody, _ := json.Marshal(webhookPayload)
	whReq := httptest.NewRequest(http.MethodPost, "/v1/telegram/webhook", bytes.NewReader(webhookBody))
	wWh := httptest.NewRecorder()
	srv.Handler().ServeHTTP(wWh, whReq)

	if wWh.Code != http.StatusOK {
		t.Fatalf("Expected 200 OK for webhook, got %d", wWh.Code)
	}

	// 4. Check Auth Status again -> Now Active!
	statusReq2 := httptest.NewRequest(http.MethodGet, "/v1/auth/status?user_id="+regResp.UserID, nil)
	wStatus2 := httptest.NewRecorder()
	srv.Handler().ServeHTTP(wStatus2, statusReq2)

	var statusResp map[string]any
	json.Unmarshal(wStatus2.Body.Bytes(), &statusResp)
	if statusResp["status"] != model.UserStatusActive {
		t.Errorf("Expected active status, got %v", statusResp["status"])
	}
}

func TestDownloadAgent(t *testing.T) {
	st := store.NewMemoryStore()
	srv := NewServer(st)

	req := httptest.NewRequest(http.MethodGet, "/v1/download/agent", nil)
	w := httptest.NewRecorder()
	srv.Handler().ServeHTTP(w, req)

	// Either serves local file (200) or redirects to GitHub (302)
	if w.Code != http.StatusOK && w.Code != http.StatusFound {
		t.Errorf("Expected 200 OK or 302 Found, got %d", w.Code)
	}
}

