package models

import "encoding/json"

// ProductGroup represents the top-level category
type ProductGroup string

const (
    GroupStructural  ProductGroup = "structural"
    GroupGuidance    ProductGroup = "guidance"
    GroupPayload     ProductGroup = "payload"
    GroupPropulsion  ProductGroup = "propulsion"
)

// ProductType represents the specific type within a group
type ProductType string

const (
    // Structural
    TypeRocketFrame        ProductType = "rocket_frame"
    TypePanelsFuselage     ProductType = "panels_fuselage"
    TypeControlFins        ProductType = "control_fins"

    // Guidance
    TypeFlightComputer     ProductType = "flight_computer"
    TypeNavSensors         ProductType = "nav_sensors"
    TypeControlActuation   ProductType = "control_actuation"
    TypeTelemetry          ProductType = "telemetry"

    // Payload
    TypeNoseCone           ProductType = "nose_cone"
    TypeCrewedCabin        ProductType = "crewed_cabin"
    TypeCargoModule        ProductType = "cargo_module"

    // Propulsion
    TypeLiquidEngine       ProductType = "liquid_engine"
    TypePropellantTank     ProductType = "propellant_tank"
    TypeRocketNozzle       ProductType = "rocket_nozzle"
)

// Product is the core product struct matching the database schema
type Product struct {
    ID          string          `json:"id"           db:"id"`
    Name        string          `json:"name"         db:"name"`
    Group       ProductGroup    `json:"group"        db:"group"`
    Type        ProductType     `json:"type"         db:"type"`
    Price       float64         `json:"price"        db:"price"`
    ImageURL    string          `json:"image_url"    db:"image_url"`
    InStock     bool            `json:"in_stock"     db:"in_stock"`
    StockCount  int             `json:"stock_count"  db:"stock_count"`
    Attributes  json.RawMessage `json:"attributes"   db:"attributes"`
    CreatedAt   string          `json:"created_at"   db:"created_at"`
    UpdatedAt   string          `json:"updated_at"   db:"updated_at"`
}

// ProductListItem is a lighter version for catalog listing pages
// — omits the full attributes blob to keep responses fast
type ProductListItem struct {
    ID         string       `json:"id"`
    Name       string       `json:"name"`
    Group      ProductGroup `json:"group"`
    Type       ProductType  `json:"type"`
    Price      float64      `json:"price"`
    ImageURL   string       `json:"image_url"`
    InStock    bool         `json:"in_stock"`
    StockCount int          `json:"stock_count"`
}

// ProductListResponse wraps a paginated list of products
type ProductListResponse struct {
    Data  []ProductListItem `json:"data"`
    Total int               `json:"total"`
    Page  int               `json:"page"`
    Limit int               `json:"limit"`
}

// ProductFilters holds optional query parameters for filtering
type ProductFilters struct {
    Group  ProductGroup `form:"group"`
    Type   ProductType  `form:"type"`
    Search string       `form:"search"`
    Page   int          `form:"page"`
    Limit  int          `form:"limit"`
}

// -----------------------------------------------------------------
// Typed attribute structs — used for validation and documentation.
// When returning a product, Attributes is raw JSON so the frontend
// can handle it dynamically. These structs are used server-side
// for any logic that needs to inspect specific fields.
// -----------------------------------------------------------------

type RocketFrameAttributes struct {
    StructuralMaterial string  `json:"structural_material"`
    Mass               float64 `json:"mass_kg"`
    Height             float64 `json:"height_m"`
    Diameter           float64 `json:"diameter_m"`
    MaxPayloadCapacity float64 `json:"max_payload_capacity_kg"`
    FlexuralStiffness  float64 `json:"flexural_stiffness_nm2"`
    TorsionalStrength  float64 `json:"torsional_strength_nm"`
}

type PanelsFuselageAttributes struct {
    Material         string  `json:"material"`
    MaxTemp          float64 `json:"max_temp_c"`
    Mass             float64 `json:"mass_kg"`
    Thickness        float64 `json:"thickness_mm"`
    CoverageArea     float64 `json:"coverage_area_m2"`
    Reusability      int     `json:"reusability_cycles"`
    ImpactResistance string  `json:"impact_resistance"`
    Aerodynamics     string  `json:"aerodynamics"`
}

type ControlFinsAttributes struct {
    Material     string  `json:"material"`
    Span         float64 `json:"span_m"`
    ChordLength  float64 `json:"chord_length_m"`
    Thickness    float64 `json:"thickness_mm"`
    NumberOfFins int     `json:"number_of_fins"`
    SweepAngle   float64 `json:"sweep_angle_deg"`
}

type FlightComputerAttributes struct {
    ProcessorSpeed         string  `json:"processor_speed_ghz"`
    MemoryCapacity         string  `json:"memory_capacity_gb"`
    RedundancyLevel        string  `json:"redundancy_level"`
    PowerConsumption       float64 `json:"power_consumption_w"`
    OperatingTempRange     string  `json:"operating_temp_range_c"`
    Weight                 float64 `json:"weight_kg"`
}

type NavSensorsAttributes struct {
    PositioningAccuracy        string  `json:"positioning_accuracy_m"`
    OrientationAccuracy        string  `json:"orientation_accuracy_deg"`
    UpdateRate                 float64 `json:"update_rate_hz"`
    SensorTypes                string  `json:"sensor_types"`
    MaxOperationalAltitude     float64 `json:"max_operational_altitude_km"`
    MaxOperationalSpeed        float64 `json:"max_operational_speed_ms"`
    Mass                       float64 `json:"mass_kg"`
    PowerConsumption           float64 `json:"power_consumption_w"`
    RedundancyLevel            string  `json:"redundancy_level"`
}

type ControlActuationAttributes struct {
    ActuationForce         float64 `json:"actuation_force_n"`
    ActuationSpeed         float64 `json:"actuation_speed_degs"`
    Precision              string  `json:"precision_mm"`
    MaximumLoad            float64 `json:"maximum_load_kg"`
    PowerConsumption       float64 `json:"power_consumption_w"`
    OperatingTempRange     string  `json:"operating_temp_range_c"`
    Mass                   float64 `json:"mass_kg"`
    RedundancyLevel        string  `json:"redundancy_level"`
}

type TelemetryAttributes struct {
    FrequencyBand          string  `json:"frequency_band"`
    TransmissionRange      float64 `json:"transmission_range_km"`
    Bandwidth              float64 `json:"bandwidth_mhz"`
    PowerConsumption       float64 `json:"power_consumption_w"`
    Mass                   float64 `json:"mass_kg"`
    AntennaType            string  `json:"antenna_type"`
    SignalModulation        string  `json:"signal_modulation"`
    RedundancyLevel        string  `json:"redundancy_level"`
    OperatingTempRange     string  `json:"operating_temp_range_c"`
}

type NoseConeAttributes struct {
    Material             string  `json:"material"`
    Length               float64 `json:"length_m"`
    BaseDiameter         float64 `json:"base_diameter_m"`
    Mass                 float64 `json:"mass_kg"`
    ThermalResistance    string  `json:"thermal_resistance"`
    PayloadCompatibility string  `json:"payload_compatibility"`
    Reusability      int     `json:"reusability_cycles"`
}

type CrewedCabinAttributes struct {
    CrewCapacity       int     `json:"crew_capacity"`
    InternalVolume     float64 `json:"internal_volume_m3"`
    Mass               float64 `json:"mass_kg"`
    LifeSupportSystems string  `json:"life_support_systems"`
    ThermalProtection  string  `json:"thermal_protection"`
    StructuralIntegrity string `json:"structural_integrity"`
}

type CargoModuleAttributes struct {
    PayloadType      string  `json:"payload_type"`
    Mass             float64 `json:"mass_kg"`
    Volume           float64 `json:"volume_m3"`
    PowerRequirements string `json:"power_requirements_w"`
    ThermalTolerance string  `json:"thermal_tolerance_c"`
}

type LiquidEngineAttributes struct {
    MaxThrust       float64 `json:"max_thrust_kn"`
    SpecificImpulse float64 `json:"specific_impulse_s"`
    BurnTime        float64 `json:"burn_time_s"`
    ChamberPressure float64 `json:"chamber_pressure_bar"`
    Weight          float64 `json:"weight_kg"`
    GimbalRange     float64 `json:"gimbal_range_deg"`
}

type PropellantTankAttributes struct {
    PropellantType          string  `json:"propellant_type"`
    TankCapacity            float64 `json:"tank_capacity_l"`
    TankMaterial            string  `json:"tank_material"`
    MaxOperatingPressure    float64 `json:"max_operating_pressure_bar"`
    InsulationType          string  `json:"insulation_type"`
    Mass                    float64 `json:"mass_kg"`
}

type RocketNozzleAttributes struct {
    NozzleType      string  `json:"nozzle_type"`
    ExpansionRatio  float64 `json:"expansion_ratio"`
    ThroatDiameter  float64 `json:"throat_diameter_m"`
    ExitDiameter    float64 `json:"exit_diameter_m"`
    Length          float64 `json:"length_m"`
    Material        string  `json:"material"`
    Mass            float64 `json:"mass_kg"`
}