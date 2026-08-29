"""Central configuration for the PreGuard pipeline."""
from dataclasses import dataclass, field


@dataclass(frozen=True)
class Settings:
    ollama_url: str = "http://localhost:11434"
    lmstudio_url: str = "http://localhost:1234"
    request_timeout_s: float = 180.0
    detect_timeout_s: float = 1.5

    supported_extensions: tuple = (
        "pdf", "pptx", "docx", "txt", "csv", "md",
        "png", "jpg", "jpeg", "tiff", "bmp", "webp", "gif",
    )

    pii_entities: tuple = ("email", "phone", "ssn", "ip", "credit_card")
    pii_labels: dict = field(default_factory=lambda: {
        "email": "Email addresses",
        "phone": "Phone numbers",
        "ssn": "SSNs",
        "ip": "IP addresses",
        "credit_card": "Credit card numbers",
    })

    system_prompt: str = (
        "You are analyzing a document that has already been scrubbed of "
        "personally identifiable information (PII) by a redaction gateway. "
        "Redacted spans appear as tags like [EMAIL], [PHONE], [SSN], "
        "[IP_ADDRESS], [CREDIT_CARD]. Never attempt to guess or reconstruct "
        "the original values behind these tags."
    )


settings = Settings()
