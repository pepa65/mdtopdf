# Client Brief: Dr Oetker & Chateau Gateaux

## Background
- Both operate as manufacturers and retail suppliers in South Africa.
- Vector Logistics handles their secondary distribution to retailers under a sell-through model (Vector purchases at a fixed price, with a later settlement on the price difference — exact mechanics to be confirmed).
- They manage deliveries to SPAR directly using Aspen Logistics and Ice Link Refrigeration Transport (cold chain).
- *Research point:* Imperial Logistics offers an open API that could enable automated delivery-note exchange. Worth investigating whether Vector, Aspen, or Ice Link have similar capabilities.

## Agreed Starting Point
The consensus from the discussion was to begin with automating **incoming and outgoing purchase order workflows through Sage** as the fastest route to realised value. Additional modules can be phased in over time.

## Strategic Positioning on Forecasting
Cargo Pilot is actively working to integrate **Prophet (from Meta/Facebook)** and **StatsForecast** into the platform to deliver effective demand tracking and forecasting with embedded machine learning models. Given that Dr Oetker and Chateau Gateaux already produce or have access to nearly all the data points required for accurate forecasting (sales, pricing, promotions, returns, stock movements, and retailer sell-out), they are in a strong position to adopt Cargo Pilot as their primary forecasting tool over time. The earlier the integration work begins, the more historical context the models can build on.

---

## Priority Modules

### 1. Purchase Order Processing
- **Incoming POs (from retailers and Vector Logistics)**
  - AI-driven extraction from emailed POs to pre-populate Cargo Pilot forms, with a human approval step.
  - Direct API integration with Vector Logistics if they can facilitate it; otherwise continue with the email-based intake model.
  - Financial sync with Sage for cost capture.
- **Outgoing POs (to suppliers and vendors)**
  - Automated PO generation triggered by reorder points and safety stock thresholds, configurable per product per location.
  - Multi-supplier management per raw material, with lead times, minimum order quantities, and supplier-specific SKUs. Reduces single-supplier risk on critical ingredients and packaging.
  - Full lifecycle tracking from draft through to Goods Received Note (GRN).
- **Vector sell-through reconciliation**
  - Workflow orchestration via N8N to monitor Vector's reported sales data and automate the price-difference settlement calculation if direct API integration is not available.

### 2. Profitability Reporting
- Track selling prices and manufacturing costs at SKU and product-line level.
- Use Cargo Pilot's Bill of Materials (BOM) cost calculation to roll up component costs, labour, and overhead per finished product.
- **BOM versioning** with revision numbers, effective dates, and scrap factor tracking per component. Supports recipe and formulation changes over time without losing historical cost context.
- Margin reporting per product line, customer, and channel.

### 3. Production Planning Tools
- **Material Requirements Planning (MRP):** Automated MRP reports generated from Sage data.
- **GRN and Delivery Note automation:** Logistics integrations to auto-generate GRNs and delivery notes in Sage for compliance and exception flagging.
- **Exception policies:** Workflow alerts when expected GRNs or delivery notes have not been received within a defined window.
- **Stock take and inventory movement audit trails:** Full history of every stock change (purchases, sales, transfers, adjustments, assemblies, returns, write-offs) for compliance and shrinkage monitoring. Supports weighted average and FIFO costing methods.
- **Consensus Forecasting:** Reconciled forecast inputs from sales, operations, and finance feeding into MRP and procurement.

### 4. Demand Forecasting
*Module under construction, target ready for testing end of June 2026. Worth trialling early to consolidate the planning stack on one platform.*

A layered, open-source forecasting approach is being built into Cargo Pilot:

- **Time-series baseline using Prophet (Meta) and StatsForecast (Nixtla):** Prophet handles seasonality, holidays, and promotional events well, while StatsForecast provides high-performance classical models (AutoARIMA, AutoETS, Holt-Winters) that scale across thousands of SKUs. Together they form a strong baseline for the seasonal patterns expected in both Dr Oetker and Chateau Gateaux product ranges.
- **Machine learning layer using Gradient Boosted Trees (XGBoost or LightGBM):** Industry standard for retail demand forecasting. Captures non-linear effects of promotions, price changes, and external factors. Applied to high-volume or high-value SKUs once richer feature data is available.
- **Consensus layer on top:** Statistical forecasts are reviewed and adjusted by sales, finance, and operations before committing to MRP and procurement.

Forecast outputs drive **MRP, reorder points, BOM purchase planning, and consensus forecasting** automatically, closing the loop between demand signals and procurement actions.

**Integration requirements:** This module requires deep integration with **POS systems** (for retailer sell-out data) and **accounting systems** (Sage, for sales, pricing, and cost data) to aggregate all input data into usable reports. Required data includes:

- Historical sales (sell-in and sell-out)
- Promotional history (mechanic, depth, duration, retailer)
- Price history (list, net, competitor where available)
- Retailer sell-out and EPOS data
- New product launches and listings changes
- Returns and write-offs (particularly relevant for short shelf-life products at Chateau Gateaux)
- Stockout flags (to avoid mistaking unavailability for zero demand)
- Weather data (relevant for seasonal categories)
- Marketing spend and media activity

[page-break]

### 5. Logistics and Despatch
- **Stock transfer orders with cold-chain cost tracking:** Manage stock movements between locations with full lifecycle tracking (draft, in-transit, partially received, completed) and per-transfer logistics cost capture including cold-chain costs. Directly relevant given the Ice Link Refrigeration involvement.
- **ZPL label printing:** Support for Zebra label printers with customisable ZPL templates for waybills, shipping labels, and invoices. Most common setup is on-site printing at 300 DPI, with optional cloud-based printing where required.
- **Customer dashboard for retailers:** Allows SPAR, Vector, and other retail customers to self-serve on order status, delivery windows, and returns. Reduces inbound support volume and improves visibility for the retailer relationship.

### 6. Dashboards and Reporting
- Cost by region.
- Rand value per case and per pallet.
- Fill rate and resource utilisation.
- Distribution centre stock levels via warehouse management system integration.

### 7. Multi-Entity Setup *(if applicable)*
If Dr Oetker and Chateau Gateaux operate as related entities and want to share infrastructure, Cargo Pilot's **master/sub-business hierarchy** supports this directly. A master business defines product catalogues, suppliers, BOMs, price lists, and carrier networks, with sub-businesses inheriting that data automatically while keeping independent transactional records (orders, inventory, POs). Configurable inheritance rules cover product ownership, customer sharing, and pricing models (uniform, regional, or transfer pricing). This needs to be confirmed with the clients before proposing.

---

## Terminology Clarifications

- **MRP (Material Requirements Planning):** A planning method that calculates which raw materials and components need to be purchased or produced, in what quantities, and by when, in order to meet manufacturing schedules. It uses the BOM, current stock, open POs, and demand forecasts as inputs.
- **Consensus Forecasting:** A demand planning process that combines forecasts from multiple internal stakeholders (sales, marketing, finance, operations) and external inputs (retailer plans) into a single agreed forecast that all functions plan against.

---

## Open Research Points
- API availability for Vector Logistics, Aspen Logistics, and Ice Link Refrigeration Transport (PO, delivery note, GRN exchange).
- Imperial Logistics API as a reference model for what's achievable with the others.
- Exact mechanics of the Vector sell-through pricing reconciliation model.
- Whether SPAR has preferred EDI or integration standards for suppliers.
- POS and EPOS data-sharing arrangements with SPAR and other retailers, since this directly affects the accuracy of demand forecasting.
- Whether Dr Oetker and Chateau Gateaux operate as related entities that would benefit from a master/sub-business setup.
