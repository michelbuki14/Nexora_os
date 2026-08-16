# Terraform Outputs - Cloud-Agnostic Configuration for Nexora OS
# ===============================================================
# All outputs are designed to be provider-agnostic where possible.
# Cloud-specific outputs are conditionally exposed based on the active mode.

# --- Core Kubernetes Outputs (always available) ---

output "namespace" {
  description = "Nexora OS Kubernetes namespace where services are deployed"
  value       = kubernetes_namespace.nexora.metadata[0].name
}

output "environment" {
  description = "Deployment environment"
  value       = var.environment
}

output "cluster_endpoint" {
  description = "Kubernetes API server endpoint (HTTPS)"
  value       = "https://${var.cluster_name}.${var.environment}.nexora-os.io"
}

output "api_gateway_service_name" {
  description = "API Gateway Kubernetes service name"
  value       = kubernetes_deployment.api-gateway.metadata[0].name
}

# --- AWS-Specific Outputs (only when AWS mode is active) ---

output "aws_ec2_instance_id" {
  description = "AWS EC2 instance ID for EKS nodes (only set when AWS mode is active)"
  value       = var.aws_enabled ? aws_ec2_instance.node.id : ""
  condition   = var.aws_enabled
}

output "aws_eks_cluster_name" {
  description = "AWS EKS cluster name (only set when AWS mode is active)"
  value       = var.aws_enabled ? aws_eks_cluster.cluster.name : ""
  condition   = var.aws_enabled
}

output "aws_region" {
  description = "AWS region in use (only set when AWS mode is active)"
  value       = var.aws_enabled ? var.aws_region : ""
  condition   = var.aws_enabled
}

# --- Azure-Specific Outputs (only when Azure mode is active) ---

output "azure_resource_group_name" {
  description = "Azure resource group name (only set when Azure mode is active)"
  value       = var.azure_enabled ? azurerm_resource_group.rg.name : ""
  condition   = var.azure_enabled
}

output "azure_aks_cluster_name" {
  description = "Azure AKS cluster name (only set when Azure mode is active)"
  value       = var.azure_enabled ? azurerm_aks_cluster.aks.name : ""
  condition   = var.azure_enabled
}

output "azure_location" {
  description = "Azure location (only set when Azure mode is active)"
  value       = var.azure_enabled ? azurerm_resource_group.rg.location : ""
  condition   = var.azure_enabled
}

# --- GCP-Specific Outputs (only when GCP mode is active) ---

output "gcp_project_id_output" {
  description = "GCP project ID (only set when GCP mode is active)"
  value       = var.gcp_enabled ? var.gcp_project_id : ""
  condition   = var.gcp_enabled
}

output "gcp_region_output" {
  description = "GCP region in use (only set when GCP mode is active)"
  value       = var.gcp_enabled ? var.gcp_region : ""
  condition   = var.gcp_enabled
}

output "gcp_gke_cluster_name" {
  description = "Google Kubernetes Engine cluster name (only set when GCP mode is active)"
  value       = var.gcp_enabled ? google_compute_cluster.gke.name : ""
  condition   = var.gcp_enabled
}

# --- Common Resource Outputs ---

output "vault_addr" {
  description = "Vault agent sidecar injector configuration"
  value       = var.enable_vault_sidecar ? "http://vault:8200" : ""
  condition   = var.enable_vault_sidecar
}

output "external_secrets_refresh_interval" {
  description = "External Secrets Operator refresh interval"
  value       = var.enable_external_secrets ? "1h" : ""
  condition   = var.enable_external_secrets
}