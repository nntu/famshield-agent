package telegram

import (
	"testing"

	"famshield-server/pkg/model"
	"famshield-server/pkg/store"
)

func TestTelegramBotActivationFlow(t *testing.T) {
	st := store.NewMemoryStore()
	svc := NewService("", "FamShieldBot", st)

	// 1. Register a user in pending state
	user, err := st.RegisterUser("parent@example.com", "hash_pw123", "guardian_dad")
	if err != nil {
		t.Fatalf("Failed to register user: %v", err)
	}

	if user.Status != model.UserStatusPendingActivation {
		t.Errorf("Expected status pending_activation, got %s", user.Status)
	}

	deepLink := svc.GetActivationDeepLink(user.ActivationToken)
	expectedLink := "https://t.me/FamShieldBot?start=" + user.ActivationToken
	if deepLink != expectedLink {
		t.Errorf("Expected deep link %s, got %s", expectedLink, deepLink)
	}

	// 2. Simulate User clicking "START" in Telegram -> bot receives /start <token>
	update := &Update{
		UpdateID: 1001,
		Message: &Message{
			MessageID: 1,
			Chat:      &Chat{ID: 123456789, Type: "private"},
			From:      &User{ID: 123456789, FirstName: "Dad", UserName: "guardian_dad"},
			Text:      "/start " + user.ActivationToken,
		},
	}

	if err := svc.HandleUpdate(update); err != nil {
		t.Fatalf("Failed to handle update: %v", err)
	}

	// 3. Verify user status became "active" and chat ID is stored
	activatedUser, found := st.GetUserByID(user.ID)
	if !found {
		t.Fatalf("User not found after activation")
	}

	if activatedUser.Status != model.UserStatusActive {
		t.Errorf("Expected status active, got %s", activatedUser.Status)
	}

	if activatedUser.TelegramChatID != 123456789 {
		t.Errorf("Expected TelegramChatID 123456789, got %d", activatedUser.TelegramChatID)
	}

	// 4. Simulate user querying /status
	statusUpdate := &Update{
		UpdateID: 1002,
		Message: &Message{
			MessageID: 2,
			Chat:      &Chat{ID: 123456789, Type: "private"},
			From:      &User{ID: 123456789, FirstName: "Dad"},
			Text:      "/status",
		},
	}

	if err := svc.HandleUpdate(statusUpdate); err != nil {
		t.Errorf("Failed to handle status command: %v", err)
	}
}
