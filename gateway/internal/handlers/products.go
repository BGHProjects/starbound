package handlers

import (
	"net/http"

	"github.com/BGHProjects/starbound/gateway/internal/db"
	"github.com/BGHProjects/starbound/gateway/internal/models"
	"github.com/gin-gonic/gin"
)

type ProductHandler struct {
    db db.DB
}

func NewProductHandler(database db.DB) *ProductHandler {
    return &ProductHandler{db: database}
}

// GetProducts godoc
// @Summary      List products
// @Description  Returns a paginated list of products, optionally filtered by group, type, or search term
// @Tags         products
// @Produce      json
// @Param        group   query     string  false  "Product group"  Enums(structural, guidance, payload, propulsion)
// @Param        type    query     string  false  "Product type"   Enums(rocket_frame, panels_fuselage, control_fins, flight_computer, nav_sensors, control_actuation, telemetry, nose_cone, crewed_cabin, cargo_module, liquid_engine, propellant_tank, rocket_nozzle)
// @Param        search  query     string  false  "Search by name"
// @Param        page    query     int     false  "Page number (default 1)"
// @Param        limit   query     int     false  "Items per page (default 20, max 100)"
// @Success      200     {object}  models.ProductListResponse
// @Failure      400     {object}  map[string]string
// @Failure      500     {object}  map[string]string
// @Router       /products [get]
func (h *ProductHandler) GetProducts(c *gin.Context) {
    var filters models.ProductFilters
    if err := c.ShouldBindQuery(&filters); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": "invalid query parameters",
        })
        return
    }

    result, err := h.db.GetProducts(filters)
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{
            "error": "failed to retrieve products",
        })
        return
    }

    c.JSON(http.StatusOK, result)
}

// GetProductByID godoc
// @Summary      Get a single product
// @Description  Returns a single product with full attributes by ID
// @Tags         products
// @Produce      json
// @Param        id   path      string  true  "Product ID"
// @Success      200  {object}  models.Product
// @Failure      400  {object}  map[string]string
// @Failure      404  {object}  map[string]string
// @Router       /products/{id} [get]
func (h *ProductHandler) GetProductByID(c *gin.Context) {
    id := c.Param("id")
    if id == "" {
        c.JSON(http.StatusBadRequest, gin.H{
            "error": "product id is required",
        })
        return
    }

    product, err := h.db.GetProductByID(id)
    if err != nil {
        c.JSON(http.StatusNotFound, gin.H{
            "error": "product not found",
        })
        return
    }

    c.JSON(http.StatusOK, product)
}

// GetProductGroups godoc
// @Summary      Get product taxonomy
// @Description  Returns the full list of product groups and their types
// @Tags         products
// @Produce      json
// @Success      200  {object}  map[string]interface{}
// @Router       /products/groups [get]
func (h *ProductHandler) GetProductGroups(c *gin.Context) {
    groups := []gin.H{
        {
            "group": "structural",
            "label": "Structural",
            "types": []gin.H{
                {"type": "rocket_frame",    "label": "Rocket Frames"},
                {"type": "panels_fuselage", "label": "Panels & Fuselage"},
                {"type": "control_fins",    "label": "Control Fins"},
            },
        },
        {
            "group": "guidance",
            "label": "Guidance",
            "types": []gin.H{
                {"type": "flight_computer",   "label": "Flight Computers"},
                {"type": "nav_sensors",       "label": "Navigation & Guidance Sensors"},
                {"type": "control_actuation", "label": "Control Actuation Systems"},
                {"type": "telemetry",         "label": "Telemetry Transmitters & Receivers"},
            },
        },
        {
            "group": "payload",
            "label": "Payload",
            "types": []gin.H{
                {"type": "nose_cone",    "label": "Nose Cones"},
                {"type": "crewed_cabin", "label": "Crewed Cabin Modules"},
                {"type": "cargo_module", "label": "Cargo Modules"},
            },
        },
        {
            "group": "propulsion",
            "label": "Propulsion",
            "types": []gin.H{
                {"type": "liquid_engine",   "label": "Liquid Rocket Engines"},
                {"type": "propellant_tank", "label": "Propellant Tanks"},
                {"type": "rocket_nozzle",   "label": "Rocket Nozzles"},
            },
        },
    }

    c.JSON(http.StatusOK, gin.H{"data": groups})
}