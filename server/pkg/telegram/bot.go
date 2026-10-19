package telegram

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"famshield-server/pkg/store"
)

// Update represents an incoming update from Telegram Bot API.
type Update struct {
	UpdateID int      `json:"update_id"`
	Message  *Message `json:"message,omitempty"`
}

type Message struct {
	MessageID int    `json:"message_id"`
	From      *User  `json:"from,omitempty"`
	Chat      *Chat  `json:"chat"`
	Text      string `json:"text,omitempty"`
}

type User struct {
	ID        int64  `json:"id"`
	FirstName string `json:"first_name"`
	UserName  string `json:"username,omitempty"`
}

type Chat struct {
	ID       int64  `json:"id"`
	Type     string `json:"type"`
	UserName string `json:"username,omitempty"`
}

type Service struct {
	token       string
	botUsername string
	store       *store.MemoryStore
	client      *http.Client
}

func NewService(token, botUsername string, st *store.MemoryStore) *Service {
	if botUsername == "" {
		botUsername = "FamShieldBot"
	}
	return &Service{
		token:       token,
		botUsername: botUsername,
		store:       st,
		client:      &http.Client{Timeout: 10 * time.Second},
	}
}

// GetActivationDeepLink generates the 1-click activation link for the Telegram bot.
func (s *Service) GetActivationDeepLink(token string) string {
	return fmt.Sprintf("https://t.me/%s?start=%s", s.botUsername, token)
}

// GetBotUsername returns the configured bot username.
func (s *Service) GetBotUsername() string {
	return s.botUsername
}

// HandleUpdate processes incoming webhooks or polled messages from Telegram.
func (s *Service) HandleUpdate(update *Update) error {
	if update == nil || update.Message == nil || update.Message.Chat == nil {
		return nil
	}

	msg := update.Message
	text := strings.TrimSpace(msg.Text)
	chatID := msg.Chat.ID
	username := ""
	if msg.From != nil {
		username = msg.From.UserName
	}

	if strings.HasPrefix(text, "/start ") {
		token := strings.TrimSpace(strings.TrimPrefix(text, "/start"))
		return s.handleActivation(token, chatID, username)
	}

	switch text {
	case "/start":
		reply := "🛡️ *Chào mừng bạn đến với FamShield Guardian Bot!*\n\n" +
			"Bot này giúp bạn kích hoạt tài khoản phụ huynh và nhận cảnh báo bảo vệ trẻ em tức thì.\n\n" +
			"👉 Để kích hoạt tài khoản, vui lòng đăng ký trên Web Console và bấm nút *'Kích hoạt qua Telegram'*."
		return s.SendMessage(chatID, reply)

	case "/status":
		user, found := s.store.GetUserByTelegramChatID(chatID)
		if !found {
			reply := "⚠️ Chat này chưa được liên kết với tài khoản FamShield nào.\n" +
				"Vui lòng đăng ký trên Web Console để liên kết."
			return s.SendMessage(chatID, reply)
		}
		reply := fmt.Sprintf("🛡️ *Trạng thái Tài khoản FamShield*\n\n"+
			"• Email: `%s`\n"+
			"• Trạng thái: *Đang hoạt động (Active)*\n"+
			"• Tenant ID: `%s`\n"+
			"• Kênh thông báo Telegram: *Đã kết nối*", user.Email, user.TenantID)
		return s.SendMessage(chatID, reply)

	default:
		reply := "🛡️ *FamShield Guardian Bot*\n\n" +
			"Gõ `/status` để xem trạng thái tài khoản bảo vệ con cái của bạn."
		return s.SendMessage(chatID, reply)
	}
}

func (s *Service) handleActivation(token string, chatID int64, username string) error {
	user, err := s.store.ActivateUserByToken(token, chatID, username)
	if err != nil {
		reply := fmt.Sprintf("⚠️ *Kích hoạt không thành công:*\n%s\n\n"+
			"Vui lòng kiểm tra lại link kích hoạt trên Web Console hoặc liên hệ hỗ trợ.", err.Error())
		return s.SendMessage(chatID, reply)
	}

	reply := fmt.Sprintf("🎉 *KÍCH HOẠT TÀI KHOẢN FAMSHIELD THÀNH CÔNG!*\n\n"+
		"Xin chào *%s*, tài khoản phụ huynh của bạn đã được kết nối với Telegram!\n\n"+
		"🛡️ *Các tính năng bảo vệ tức thì đã kích hoạt:*\n"+
		"• Nhận cảnh báo khi con truy cập web nguy hiểm/nội dung cấm\n"+
		"• Nhận thông báo khi con chạm hạn mức thời gian dùng máy\n"+
		"• Phê duyệt yêu cầu xin thêm giờ với 1 chạm trực tiếp\n\n"+
		"🌐 *Đăng nhập Web Console:* https://admin.famshield.cloud", user.Email)

	log.Printf("Successfully activated user %s via Telegram ChatID %d (@%s)", user.Email, chatID, username)
	return s.SendMessage(chatID, reply)
}

// SendMessage sends a formatted Markdown message to the specified Telegram Chat.
func (s *Service) SendMessage(chatID int64, text string) error {
	if s.token == "" {
		log.Printf("[Telegram Mock] To ChatID %d: %s", chatID, text)
		return nil
	}

	url := fmt.Sprintf("https://api.telegram.org/bot%s/sendMessage", s.token)
	payload := map[string]any{
		"chat_id":    chatID,
		"text":       text,
		"parse_mode": "Markdown",
	}

	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	resp, err := s.client.Post(url, "application/json", bytes.NewReader(body))
	if err != nil {
		log.Printf("Warning: failed to send Telegram message: %v", err)
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		log.Printf("Warning: Telegram API returned status %d", resp.StatusCode)
	}
	return nil
}
