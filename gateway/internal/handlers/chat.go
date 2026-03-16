package handlers

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

const ragServiceURL = "http://localhost:8001"

type ChatRequest struct {
	Query     string  `json:"query"      binding:"required"`
	SessionID *string `json:"session_id"`
}

type ChatResponse struct {
	Answer  string   `json:"answer"`
	Sources []string `json:"sources"`
}

var ragClient = &http.Client{
	Timeout: 60 * time.Second,
}

// ProxyChat forwards the chat request to the RAG service
func ProxyChat(c *gin.Context) {
	var req ChatRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	body, err := json.Marshal(req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to encode request"})
		return
	}

	resp, err := ragClient.Post(
		fmt.Sprintf("%s/api/chat", ragServiceURL),
		"application/json",
		bytes.NewReader(body),
	)
	if err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"error": fmt.Sprintf("RAG service unavailable: %s", err.Error()),
		})
		return
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to read RAG response"})
		return
	}

	var chatResp ChatResponse
	if err := json.Unmarshal(respBody, &chatResp); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to decode RAG response"})
		return
	}

	c.JSON(resp.StatusCode, chatResp)
}