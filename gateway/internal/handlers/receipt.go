package handlers

import (
	"bytes"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/gin-gonic/gin"
	"github.com/go-pdf/fpdf"
)

type ReceiptHandler struct {
	orders *db.OrderStore
}

func NewReceiptHandler(orders *db.OrderStore) *ReceiptHandler {
	return &ReceiptHandler{orders: orders}
}

func (h *ReceiptHandler) GetReceipt(c *gin.Context) {
	userID  := c.GetString("user_id")
	orderID := c.Param("id")

	order, err := h.orders.GetOrderByID(orderID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "order not found"})
		return
	}

	// Enforce ownership
	if order.UserID != userID {
		c.JSON(http.StatusForbidden, gin.H{"error": "access denied"})
		return
	}

	pdf := fpdf.New("P", "mm", "A4", "")
	pdf.AddPage()
	pdf.SetMargins(20, 20, 20)

	// ── Header ────────────────────────────────────────────────
	pdf.SetFont("Helvetica", "B", 24)
	pdf.SetTextColor(244, 104, 26) // orange
	pdf.CellFormat(0, 12, "STARBOUND", "", 1, "C", false, 0, "")

	pdf.SetFont("Helvetica", "", 10)
	pdf.SetTextColor(122, 138, 170) // muted
	pdf.CellFormat(0, 6, "Rocket Parts Marketplace", "", 1, "C", false, 0, "")
	pdf.CellFormat(0, 6, "starbound.dev", "", 1, "C", false, 0, "")

	pdf.Ln(8)

	// Horizontal rule
	pdf.SetDrawColor(30, 46, 80) // border colour
	pdf.SetLineWidth(0.5)
	pdf.Line(20, pdf.GetY(), 190, pdf.GetY())
	pdf.Ln(6)

	// ── Order info ────────────────────────────────────────────
	pdf.SetFont("Helvetica", "B", 11)
	pdf.SetTextColor(32, 37, 45) // dark grey
	pdf.CellFormat(0, 8, "ORDER RECEIPT", "", 1, "L", false, 0, "")
	pdf.Ln(2)

	pdf.SetFont("Helvetica", "", 9)
	pdf.SetTextColor(122, 138, 170)

	pdf.CellFormat(40, 6, "Order ID:", "", 0, "L", false, 0, "")
	pdf.SetTextColor(32, 37, 45)
	pdf.SetFont("Helvetica", "B", 9)
	pdf.CellFormat(0, 6, order.ID, "", 1, "L", false, 0, "")

	pdf.SetFont("Helvetica", "", 9)
	pdf.SetTextColor(122, 138, 170)
	pdf.CellFormat(40, 6, "Order date:", "", 0, "L", false, 0, "")
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(0, 6, formatReceiptDate(order.CreatedAt), "", 1, "L", false, 0, "")

	pdf.SetTextColor(122, 138, 170)
	pdf.CellFormat(40, 6, "Status:", "", 0, "L", false, 0, "")
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(0, 6, formatReceiptStatus(string(order.Status)), "", 1, "L", false, 0, "")

	pdf.Ln(6)
	pdf.SetDrawColor(30, 46, 80)
	pdf.Line(20, pdf.GetY(), 190, pdf.GetY())
	pdf.Ln(6)

	// ── Shipping address ──────────────────────────────────────
	pdf.SetFont("Helvetica", "B", 10)
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(0, 7, "SHIPPING ADDRESS", "", 1, "L", false, 0, "")
	pdf.Ln(1)

	pdf.SetFont("Helvetica", "", 9)
	pdf.SetTextColor(122, 138, 170)

	addr := order.ShippingAddress
	if addr.FacilityName != "" {
		pdf.CellFormat(0, 5, addr.FacilityName, "", 1, "L", false, 0, "")
	}
	if addr.SiteCode != "" {
		pdf.CellFormat(0, 5, fmt.Sprintf("Site: %s", addr.SiteCode), "", 1, "L", false, 0, "")
	}
	pdf.CellFormat(0, 5, addr.AddressLine1, "", 1, "L", false, 0, "")
	if addr.AddressLine2 != "" {
		pdf.CellFormat(0, 5, addr.AddressLine2, "", 1, "L", false, 0, "")
	}
	pdf.CellFormat(0, 5, fmt.Sprintf("%s, %s", addr.City, addr.PostalCode), "", 1, "L", false, 0, "")
	pdf.CellFormat(0, 5, addr.Country, "", 1, "L", false, 0, "")

	pdf.Ln(6)
	pdf.SetDrawColor(30, 46, 80)
	pdf.Line(20, pdf.GetY(), 190, pdf.GetY())
	pdf.Ln(6)

	// ── Items ─────────────────────────────────────────────────
	pdf.SetFont("Helvetica", "B", 10)
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(0, 7, "ITEMS ORDERED", "", 1, "L", false, 0, "")
	pdf.Ln(1)

	// Table header
	pdf.SetFont("Helvetica", "B", 9)
	pdf.SetTextColor(122, 138, 170)
	pdf.CellFormat(90, 6, "Product", "B", 0, "L", false, 0, "")
	pdf.CellFormat(30, 6, "Type", "B", 0, "L", false, 0, "")
	pdf.CellFormat(20, 6, "Qty", "B", 0, "C", false, 0, "")
	pdf.CellFormat(30, 6, "Line total", "B", 1, "R", false, 0, "")
	pdf.Ln(1)

	// Table rows
	pdf.SetFont("Helvetica", "", 9)
	for _, item := range order.Items {
		pdf.SetTextColor(32, 37, 45)
		pdf.CellFormat(90, 6, truncate(item.ProductName, 45), "", 0, "L", false, 0, "")
		pdf.SetTextColor(122, 138, 170)
		pdf.CellFormat(30, 6, item.ProductType, "", 0, "L", false, 0, "")
		pdf.SetTextColor(32, 37, 45)
		pdf.CellFormat(20, 6, fmt.Sprintf("%d", item.Quantity), "", 0, "C", false, 0, "")
		pdf.SetTextColor(244, 104, 26)
		pdf.CellFormat(30, 6, formatMoney(item.LineTotal), "", 1, "R", false, 0, "")
	}

	pdf.Ln(4)
	pdf.SetDrawColor(30, 46, 80)
	pdf.Line(20, pdf.GetY(), 190, pdf.GetY())
	pdf.Ln(4)

	// ── Totals ────────────────────────────────────────────────
	pdf.SetFont("Helvetica", "", 9)
	pdf.SetTextColor(122, 138, 170)

	pdf.CellFormat(150, 6, "Subtotal", "", 0, "R", false, 0, "")
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(20, 6, formatMoney(order.Subtotal), "", 1, "R", false, 0, "")

	pdf.SetTextColor(122, 138, 170)
	pdf.CellFormat(150, 6, "Shipping", "", 0, "R", false, 0, "")
	pdf.SetTextColor(32, 37, 45)
	pdf.CellFormat(20, 6, formatMoney(order.ShippingCost), "", 1, "R", false, 0, "")

	pdf.Ln(2)
	pdf.SetFont("Helvetica", "B", 11)
	pdf.SetTextColor(122, 138, 170)
	pdf.CellFormat(150, 8, "Total", "", 0, "R", false, 0, "")
	pdf.SetTextColor(244, 104, 26)
	pdf.CellFormat(20, 8, formatMoney(order.Total), "", 1, "R", false, 0, "")

	pdf.Ln(8)
	pdf.SetDrawColor(30, 46, 80)
	pdf.Line(20, pdf.GetY(), 190, pdf.GetY())
	pdf.Ln(6)

	// ── Footer ────────────────────────────────────────────────
	pdf.SetFont("Helvetica", "I", 8)
	pdf.SetTextColor(58, 78, 112) // dim
	pdf.MultiCell(0, 5,
		"This is a portfolio project receipt generated for demonstration purposes only. "+
			"No real transaction has taken place. "+
			"Starbound is not a real company.",
		"", "C", false)

	// Write PDF to buffer
	var buf bytes.Buffer
	if err := pdf.Output(&buf); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to generate PDF"})
		return
	}

	filename := fmt.Sprintf("starbound-receipt-%s.pdf", orderID[:8])
	c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=%s", filename))
	c.Header("Content-Type", "application/pdf")
	c.Header("Content-Length", fmt.Sprintf("%d", buf.Len()))
	c.Data(http.StatusOK, "application/pdf", buf.Bytes())
}

// ── Helpers ───────────────────────────────────────────────────────

func formatMoney(amount float64) string {
	if amount >= 1_000_000 {
		return fmt.Sprintf("$%.1fM", amount/1_000_000)
	} else if amount >= 1_000 {
		return fmt.Sprintf("$%.0fK", amount/1_000)
	}
	return fmt.Sprintf("$%.2f", amount)
}

func formatReceiptDate(ts string) string {
	t, err := time.Parse(time.RFC3339, ts)
	if err != nil {
		if len(ts) >= 10 {
			return ts[:10]
		}
		return ts
	}
	return t.Format("2 January 2006")
}

func formatReceiptStatus(status string) string {
	words := strings.Split(status, "_")
	for i, w := range words {
		if len(w) > 0 {
			words[i] = strings.ToUpper(w[:1]) + w[1:]
		}
	}
	return strings.Join(words, " ")
}

func truncate(s string, max int) string {
	if len(s) <= max {
		return s
	}
	return s[:max-1] + "…"
}