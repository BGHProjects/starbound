package handlers

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

const cvServiceURL = "http://localhost:8002"

type RefundResponse struct {
	Valid    bool    `json:"valid"`
	OrderID  *string `json:"order_id"`
	Reason   string  `json:"reason"`
}

var cvClient = &http.Client{
	Timeout: 60 * time.Second,
}

// ProxyRefund forwards the PDF upload and order_id to the CV service
func ProxyRefund(c *gin.Context) {
	// Get order_id from the form
	if err := c.Request.ParseMultipartForm(32 << 20); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse form"})
		return
	}
	orderID := c.Request.FormValue("order_id")
	if orderID == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "order_id is required"})
		return
	}

	file, header, err := c.Request.FormFile("file")
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no file uploaded"})
		return
	}
	defer file.Close()

	fileBytes, err := io.ReadAll(file)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to read file"})
		return
	}

	// Rebuild multipart form for forwarding to CV service
	var buf bytes.Buffer
	writer := multipart.NewWriter(&buf)

	// Add order_id field
	if err := writer.WriteField("order_id", orderID); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to write order_id"})
		return
	}

	// Add file
	part, err := writer.CreateFormFile("file", header.Filename)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create form"})
		return
	}

	if _, err = part.Write(fileBytes); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to write file"})
		return
	}
	writer.Close()

	req, err := http.NewRequest(
		"POST",
		fmt.Sprintf("%s/api/refund/validate", cvServiceURL),
		&buf,
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to build request"})
		return
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())

	resp, err := cvClient.Do(req)
	if err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"error": fmt.Sprintf("CV service unavailable: %s", err.Error()),
		})
		return
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to read CV response"})
		return
	}

	var refundResp RefundResponse
	if err := json.Unmarshal(respBody, &refundResp); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to decode CV response"})
		return
	}

	c.JSON(resp.StatusCode, refundResp)
}