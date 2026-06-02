pub fn classify_domain(text: &str) -> (&'static str, &'static str) {
    let text_lower = text.to_lowercase();
    
    let finance_keywords = vec![
        "balance sheet", "income", "revenue", "profit", "cash flow", "equity", "asset", 
        "fiscal", "tax", "audit", "quarterly", "earnings", "dividend", "financial", 
        "ebitda", "amortization", "depreciation", "ledger", "accounting", "liability"
    ];

    let supply_planning_keywords = vec![
        "inventory", "demand", "supply", "lead time", "forecast", "safety stock", "warehouse", 
        "logistics", "shipment", "capacity", "sku", "replenishment", "backorder", "stockout", 
        "distribution", "mrp", "reorder point", "fulfillment", "freight", "carrier"
    ];

    let manufacturing_keywords = vec![
        "factory", "production", "assembly", "shop floor", "machinery", "tooling", "yield", 
        "quality control", "maintenance", "throughput", "raw material", "oee", "lean", 
        "six sigma", "kaizen", "downtime", "manufacturing", "wip", "work in progress", "bom"
    ];

    let procurement_keywords = vec![
        "purchase order", "po", "supplier", "vendor", "rfp", "contract", "quote", "invoice", 
        "sourcing", "spend", "procurement", "bidding", "requisition", "rfq", "procure", 
        "supplier relationship", "commercial terms", "price sheet"
    ];

    let legal_keywords = vec![
        "agreement", "contract", "party", "liability", "clause", "jurisdiction", "indemnify", 
        "warranty", "breach", "litigation", "intellectual property", "nda", "disclosure", 
        "confidentiality", "arbitration", "governing law", "termination", "force majeure", 
        "statute", "covenant"
    ];

    let hr_keywords = vec![
        "employee", "payroll", "benefits", "recruitment", "hiring", "performance review", 
        "talent", "onboarding", "training", "compensation", "policy", "resume", "cv", 
        "human resources", "hr", "wellness", "absenteeism", "retire", "leave of absence"
    ];

    let tech_doc_keywords = vec![
        "api", "database", "configure", "architecture", "installation", "system", "deployment", 
        "hardware", "software", "user manual", "developer", "sdk", "endpoint", "git", "docker", 
        "kubernetes", "server", "json", "config", "command line", "terminal"
    ];

    let research_keywords = vec![
        "abstract", "methodology", "results", "discussion", "citation", "dataset", "experiment", 
        "hypothesis", "literature review", "statistics", "conclusion", "peer-reviewed", 
        "scientific", "journal", "academic", "theory", "empirical", "model", "analysis"
    ];

    let mut scores = vec![
        ("Finance", "FinanceAgent", 0),
        ("Supply Planning", "SupplyPlanningAgent", 0),
        ("Manufacturing", "ManufacturingAgent", 0),
        ("Procurement", "ProcurementAgent", 0),
        ("Legal", "LegalAgent", 0),
        ("HR", "HRAgent", 0),
        ("Technical Documentation", "TechnicalDocAgent", 0),
        ("Research", "ResearchAgent", 0),
    ];

    for item in &mut scores {
        let keywords = match item.0 {
            "Finance" => &finance_keywords,
            "Supply Planning" => &supply_planning_keywords,
            "Manufacturing" => &manufacturing_keywords,
            "Procurement" => &procurement_keywords,
            "Legal" => &legal_keywords,
            "HR" => &hr_keywords,
            "Technical Documentation" => &tech_doc_keywords,
            "Research" => &research_keywords,
            _ => continue,
        };

        let mut count = 0;
        for kw in keywords {
            count += text_lower.matches(kw).count();
        }
        item.2 = count;
    }

    let mut best_class = "Technical Documentation";
    let mut best_agent = "TechnicalDocAgent";
    let mut max_score = 0;

    for (class, agent, score) in scores {
        if score > max_score {
            max_score = score;
            best_class = class;
            best_agent = agent;
        }
    }

    if max_score == 0 {
        if text_lower.contains("class ") || text_lower.contains("fn ") || text_lower.contains("import ") {
            best_class = "Technical Documentation";
            best_agent = "TechnicalDocAgent";
        } else if text_lower.contains("abstract") || text_lower.contains("references") {
            best_class = "Research";
            best_agent = "ResearchAgent";
        }
    }

    (best_class, best_agent)
}
