# Terraform Variables - Cloud-Agnostic Configuration for Nexora OS
# ==================================================================
# All variables are designed to work across multiple cloud providers.
# Values can be set via:
# - terraform.tfvars file
# - Environment variables
# - CI/CD pipeline parameters
# - Interactive prompt (terraform init)

# --- Provider Mode Variables ---

# Determine which cloud provider to use
# When set to "true", the corresponding provider is enabled
# When set to "false" or empty, that provider is skipped
# When all are empty/default, Kubernetes-only mode is used

variable "aws_mode" {
  description = "Enable AWS/EKS deployment mode"
  type        = string
  default     = ""
}

variable "azure_mode" {
  description = "Enable Azure/AKS deployment mode"
  type        = string
  default     = ""
}

variable "gcp_mode" {
  description = "Enable GCP/GKE deployment mode"
  type        = string
  default     = ""
}

# --- Environment Variables ---

variable "environment" {
  description = "Deployment environment (development, staging, production)"
  type        = string
  default     = "development"
}

variable "cluster_name" {
  description = "Kubernetes cluster name"
  type        = string
  default     = "nexora-cluster"
}

# --- AWS Specific Variables ---

variable "aws_region" {
  description = "AWS region for EKS deployment"
  type        = string
  default     = "us-east-1"
}

variable "aws_account_id" {
  description = "AWS account ID for EKS deployment"
  type        = string
  default     = "123456789012"
}

variable "vpc_id" {
  description = "Existing VPC ID to deploy into (for existing infrastructure)"
  type        = string
  default     = ""
}

variable "subnet_ids" {
  description = "Subnet IDs for EKS nodes (comma-separated list)"
  type        = list(string)
  default     = []
}

# --- Azure Specific Variables ---

variable "azure_subscription_id" {
  description = "Azure subscription ID for AKS deployment"
  type        = string
  default     = "00000000-0000-0000-0000-000000000000"
}

variable "azure_tenant_id" {
  description = "Azure tenant ID for AKS deployment"
  type        = string
  default     = "00000000-0000-0000-0000-000000000000"
}

variable "azure_resource_group" {
  description = "Existing Azure resource group to deploy into"
  type        = string
  default     = ""
}

# --- GCP Specific Variables ---

variable "gcp_project_id" {
  description = "GCP project ID for GKE deployment"
  type        = string
  default     = "nexora-os-dev"
}

variable "gcp_region" {
  description = "GCP region for GKE deployment"
  type        = string
  default     = "us-central1"
}

variable "gcp_network" {
  description = "Existing GCP network to deploy into"
  type        = string
  default     = ""
}

# --- Kubernetes Variables ---

variable "node_count" {
  description = "Initial node count for Kubernetes cluster"
  type        = number
  default     = 2
}

variable "node_size" {
  description = "Node size / instance type"
  type        = string
  default     = "standard-medium"
}

variable "node_selector" {
  description = "Node selector labels (map format)"
  type        = map(string)
  default     = {}
}

variable "tolerations" {
  description = "Tolerations for pod scheduling (list format)"
  type        = list(map(string))
  default     = []
}

variable "affinity" {
  description = "Affinity rules for pod scheduling (map format)"
  type        = map(string)
  default     = {}
}

# --- Image Variables ---

# Container images used across all deployments
# These are built per-platform and pushed to supported registries

variable "api_gateway_image" {
  description = "API Gateway container image"
  type        = string
  default     = "nexora-services:latest"
}

variable "workforce_service_image" {
  description = "Workforce Service container image"
  type        = string
  default     = "nexora-services:latest"
}

variable "audit_service_image" {
  description = "Audit Service container image"
  type        = string
  default     = "nexora-services:latest"
}

variable "tenant_service_image" {
  description = "Tenant Service container image"
  type        = string
  default     = "nexora-services:latest"
}

variable "migrate_service_image" {
  description = "Migrate Service container image"
  type        = string
  default     = "nexora-services:latest"
}

# --- Tags and Labeling ---

variable "tags" {
  description = "Common tags applied to all resources"
  type = map(string)
  default = {
    Environment = var.environment
    Project     = "nexora-os"
    ManagedBy   = "terraform"
  }
}

# --- Feature Flags ---

variable "enable_grafana" {
  description = "Enable Grafana dashboard deployment"
  type        = bool
  default     = true
}

variable "enable_prometheus" {
  description = "Enable Prometheus metrics deployment"
  type        = bool
  default     = true
}

variable "enable_external_secrets" {
  description = "Enable External Secrets Operator integration"
  type        = bool
  default     = true
}

variable "enable_vault_sidecar" {
  description = "Enable Vault sidecar injector for secret injection"
  type        = bool
  default     = false
}