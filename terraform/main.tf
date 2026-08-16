# Terraform Cloud-Agnostic Configuration for Nexora OS
# ======================================================
# This configuration supports multiple cloud providers (AWS, Azure, GCP, OnPrem)
# by using backend-agnostic configuration and environment-specific overrides.
# The goal is to avoid vendor lock-in and enable multi-cloud deployments.

# Terraform required providers
terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }
}

# Provider configuration - selected based on CONTAINER_ENV
# Default to Kubernetes for maximum portability
provider "kubernetes" {
  # Provider will be selected based on environment
  # In AWS: uses EKS cluster
  # In Azure: uses AKS cluster
  # In GCP: uses GKE cluster
  # On-prem: uses k0s or k3s

  # The cluster name and region are set via variables
  # kubeconfig is sourced from environment-specific secrets
}

# --- AWS Provider (conditionally enabled) ---
# Only configure AWS when AWS_MODE is set to "true"
# This allows running the same Terraform config on other cloud providers
# without the AWS provider being loaded.

# Get AWS region from environment - defaults to us-east-1 if not set
variable "aws_region" {
  description = "AWS region for EKS deployment"
  type        = string
  default     = "us-east-1"
}

# Get AWS account ID - used for cross-account operations
variable "aws_account_id" {
  description = "AWS account ID for EKS deployment"
  type        = string
  default     = "123456789012"
}

# Conditionally enable AWS provider
locals {
  aws_enabled = contains(tolist(var.aws_mode), "true")
}

provider "aws" {
  region = var.aws_region
  # Skip if AWS is not the target platform
  # The kubernetes provider is always available as fallback
  # when aws_enabled is false, the AWS provider module won't be loaded
  # but we still declare it for module compatibility
  # In practice, the kubernetes provider handles the actual deployment
}

# --- Azure Provider (conditionally enabled) ---
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

variable "azure_client_id" {
  description = "Azure client ID for AKS deployment"
  type        = string
  default     = ""
}

variable "azure_client_secret" {
  description = "Azure client secret for AKS deployment"
  type        = string
  default     = ""
}

# Conditionally enable Azure provider
local {
  azure_enabled = contains(tolist(var.azure_mode), "true")
}

provider "azurerm" {
  subscription_id = var.azure_subscription_id
  tenant_id       = var.azure_tenant_id
  client_id       = var.azure_client_id
  client_secret   = var.azure_client_secret
  # feature {
  #   # Currently registering all the resource types
  #   # required for the modules we use
  #   register_microsoft_all = true
  # }
}

# --- GCP Provider (conditionally enabled) ---
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

# Conditionally enable GCP provider
local {
  gcp_enabled = contains(tolist(var.gcp_mode), "true")
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

# --- Common Configuration ---

# Environment name (development, staging, production)
variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "development"
}

# Kubernetes cluster name
variable "cluster_name" {
  description = "Kubernetes cluster name"
  type        = string
  default     = "nexora-cluster"
}

# Node pool configuration
variable "node_count" {
  description = "Initial node count"
  type        = number
  default     = 2
}

variable "node_size" {
  description = "Node size (instance type)"
  type        = string
  default     = "standard-medium"
}

# Tags for all resources
variable "tags" {
  description = "Common tags for all resources"
  type = map(string)
  default = {
    Environment = var.environment
    Project     = "nexora-os"
    ManagedBy   = "terraform"
  }
}

# --- Deployment Target Selection ---

# Determine which cloud provider to use based on environment variables
# Priority: explicit mode flags > detected from infrastructure

# Mode can be set via:
# - Terraform variable: aws_mode, azure_mode, gcp_mode
# - Environment variables: AWS_MODE, AZURE_MODE, GCP_MODE
# - If none set, defaults to Kubernetes-only

# Kubernetes deployment (always available)
# This manifest works across any Kubernetes cluster regardless of cloud provider

resource "kubernetes_namespace" "nexora" {
  metadata {
    name      = "nexora-${var.environment}"
    labels = {
      project = "nexora-os"
      environment = var.environment
    }
  }
}

# Deployment with cloud-agnostic configuration
resource "kubernetes_deployment" "api-gateway" {
  metadata {
    name      = "nexora-api-gateway"
    namespace = kubernetes_namespace.nexora.metadata[0]
    labels = {
      app = "nexora-api-gateway"
      environment = var.environment
    }
  }

  # The spec uses container images that are built per-platform
  # and pushed to registries that support all cloud providers
  # (ECR, ACR, GCR, or self-hosted registry)
  spec {
    replicas = var.node_count

    selector {
      match_labels = {
        app = "nexora-api-gateway"
      }
    }

    template {
      metadata {
        labels = {
          app = "nexora-api-gateway"
        }
      }

      spec {
        # Service account for the application
        # Created separately via Kubernetes RBAC
        # automountServiceAccountToken is set to false for security
        # and the service account is bound to the minimum required permissions
        automountServiceAccountToken = false

        # Container image - pulled from registry that supports all clouds
        container {
          # Image repository and tag are set per-environment
          # Images are built and pushed to:
          # - AWS: ghcr.io/michelbuki14/aos or ECR
          # - Azure: mcr.microsoft.com/azure-dev/integraz or ACR
          # - GCP: us-docker.pkg.dev/gcp-project/images or GCR
          # - On-prem: self-hosted registry
          image = var.api_gateway_image
        }

        # Resource limits and requests
        resources {
          limit {
            cpu    = "500m"
            memory = "512Mi"
          }
          request {
            cpu    = "100m"
            memory = "128Mi"
          }
        }

        # Container ports
        container_port {
          container_port = 3000
        }

        # Liveness probe
        liveness_probe {
          http_get {
            path = "/health/live"
            port = 3000
          }
          initial_delay_seconds = 10
          period_seconds = 10
          timeout_seconds = 3
          failure_threshold = 6
        }

        # Readiness probe
        readiness_probe {
          http_get {
            path = "/health/ready"
            port = 3000
          }
          initial_delay_seconds = 10
          period_seconds = 10
          timeout_seconds = 3
          failure_threshold = 6
        }

        # Environment variables from config
        # All env vars use the AOS_ prefix convention
        # Sensitive values (DB URLs, keys) are referenced via secretKeyRef
        # from secrets managed by External Secrets Operator or Vault
        env {
          # Service identification
          name  = "AOS_SERVICE__ENVIRONMENT"
          value = var.environment

          # Server configuration
          name  = "AOS_SERVER__HOST"
          value = "0.0.0.0"

          name  = "AOS_SERVER__PORT"
          value = "3000"

          # Database configuration - referenced from secrets
          # In production, these are populated by ESO from cloud secret managers
          # or from Vault via the sidecar injector
          # name  = "AOS_DATABASE__URL"
          # value = var.database_url

          # Redis configuration
          # name  = "AOS_REDIS__URL"
          # value = var.redis_url

          # Authentication - Keycloak configuration
          # These are populated from Vault or cloud secret managers
          # name  = "AOS_AUTH__JWKS_URL"
          # value = var.keycloak_jwks_url

          # name  = "AOS_AUTH__ISSUER"
          # value = var.keycloak_issuer

          # name  = "AOS_AUTH__AUDIENCE"
          # value = var.keycloak_audience

          # Tracing - OpenTelemetry
          # name  = "AOS_TRACING__OTLP_ENDPOINT"
          # value = var.otlp_endpoint

          # name  = "AOS_TRACING__SAMPLE_RATE"
          # value = "0.1"

          # S3/MinIO configuration
          # name  = "AOS_S3__ENDPOINT"
          # value = var.s3_endpoint

          # name  = "AOS_S3__ACCESS_KEY_ID"
          # (referenced via secretKeyRef, not inline)

          # name  = "AOS_S3__SECRET_ACCESS_KEY"
          # (referenced via secretKeyRef, not inline)
        }

        # Node selectors and tolerations
        # Can be overridden per-environment
        node_selector = var.node_selector

        tolerations = var.tolerations

        # Affinity rules
        affinity = var.affinity
      }
    }
  }
}

# --- Outputs ---

# Output the cluster endpoint - this varies by cloud provider
# but the format is consistent for Kubernetes consumers
output "cluster_endpoint" {
  description = "Kubernetes API server endpoint"
  value       = "https://${var.cluster_name}.${var.environment}.nexora-os.io"
}

# Output the namespace for resources
output "namespace" {
  description = "Nexora OS Kubernetes namespace"
  value       = kubernetes_namespace.nexora.metadata[0].name
}

# Output the deployment name
output "api_gateway_deployment" {
  description = "API Gateway deployment name"
  value       = kubernetes_deployment.api-gateway.metadata[0].name
}

# Output the environment
output "environment" {
  description = "Deployment environment"
  value       = var.environment
}

# Output the Terraform workspace/provider being used
output "terraform_workspace" {
  description = "Current Terraform workspace"
  value       = terraform.workspace
}

# Conditional outputs based on provider
# Only show AWS-specific outputs when AWS is the target
output "aws_cluster_name" {
  description = "AWS EKS cluster name (only set when AWS mode is active)"
  value       = var.aws_enabled ? var.cluster_name : ""
  condition   = var.aws_enabled
}

output "azure_resource_group" {
  description = "Azure resource group name (only set when Azure mode is active)"
  value       = var.azure_enabled ? "nexora-rg-${var.environment}" : ""
  condition   = var.azure_enabled
}

output "gcp_project" {
  description = "GCP project ID (only set when GCP mode is active)"
  value       = var.gcp_enabled ? var.gcp_project_id : ""
  condition   = var.gcp_enabled
}