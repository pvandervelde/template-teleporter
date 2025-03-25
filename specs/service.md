# GitHub Template Sync Application Specification

## Purpose

The GitHub Template Sync Application automates the maintenance of issue and pull request templates across multiple GitHub
repositories. It ensures that templates are consistent across repositories of the same category (e.g., SaaS applications,
libraries) while respecting customizations made by repository maintainers.

The application monitors a master template repository, and when template updates occur, it automatically creates pull
requests to update templates in the appropriate target repositories. This reduces manual maintenance work, ensures
consistency, and provides a clear audit trail of template changes.

## Use Cases

### 1. Keeping Templates Up-to-Date

**As a** platform engineering team member
**I want** templates in all repositories to automatically update when the master template changes
**So that** we maintain consistent standards across our organization

### 2. Template Customization

**As a** repository maintainer
**I want** the system to respect my customized templates
**So that** I can adapt templates to my specific project needs without losing them on updates

### 3. Adding New Repositories

**As a** developer creating a new repository
**I want** the appropriate templates to be automatically applied
**So that** I don't have to manually create or copy templates

### 4. Template Governance

**As a** compliance officer
**I want** a clear audit trail of template changes
**So that** I can verify when and how templates were updated

### 5. Template Distribution by Project Type

**As a** platform engineering team
**I want** to define different templates for different repository types
**So that** each project category has the most appropriate templates

### 6. Manual Template Synchronization

**As a** DevOps engineer
**I want** to trigger template synchronization manually via CLI
**So that** I can update templates without waiting for a push event

## Master Template Repository Structure

The master repository should be organized as follows:

```
/
├── README.md                   # Documentation about the template system
├── config.yaml                 # Configuration file mapping templates to repository categories
└── templates/                  # Root directory for all templates
    ├── saas/                   # Templates for SaaS applications
    │   ├── PULL_REQUEST_TEMPLATE.md
    │   ├── ISSUE_TEMPLATE.md
    │   └── .github/
    │       └── ISSUE_TEMPLATE/
    │           ├── bug_report.md
    │           ├── feature_request.md
    │           └── regression.md
    ├── library/                # Templates for software libraries
    │   ├── PULL_REQUEST_TEMPLATE.md
    │   ├── ISSUE_TEMPLATE.md
    │   └── .github/
    │       └── ISSUE_TEMPLATE/
    │           ├── bug_report.md
    │           ├── feature_request.md
    │           └── documentation.md
    └── common/                 # Templates that apply to all repositories
        ├── CODE_OF_CONDUCT.md
        └── CONTRIBUTING.md
```

## Configuration File

The `config.yaml` file defines the mapping between repository categories and templates. Example structure:

```yaml
categories:
  - name: saas
    description: "Software as a Service applications"
    topics: ["saas", "webapp", "service"]
    path: templates/saas

  - name: library
    description: "Reusable software libraries and packages"
    topics: ["library", "sdk", "package"]
    path: templates/library

  - name: common
    description: "Templates that apply to all repositories"
    topics: ["*"]
    path: templates/common

# Optional global settings
settings:
  # If true, the system won't update templates that have been modified locally
  respect_customizations: true

  # Default branch to create PRs against (if not specified, uses the repo's default branch)
  default_target_branch: main

  # PR settings
  pull_request:
    title_prefix: "[Template Sync]"
    reviewers: ["@platform-team"]
```

## Application Repository Structure

The application should be organized as follows:

```
/
├── README.md                   # Documentation for setup and usage
├── CONTRIBUTING.md             # Contribution guidelines
├── LICENSE                     # License information
├── Cargo.toml                  # Rust package manifest
├── .github/                    # CI/CD workflows and templates for this repo
│   └── workflows/
│       ├── build.yml           # CI workflow for building and testing
│       └── deploy.yml          # CD workflow for deployment
├── crates/                     # Rust source code
│   ├── aws-lambda
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # Application entry point
│   ├── azure-function
│   │   ├── Cargo.toml
│   │   ├── src/
│   │       ├── main.rs          # Application entry point
│   ├── cli                      # CLI-specific functionality
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── args.rs          # Command-line argument parsing
│   │       ├── command.rs       # CLI command implementations
│   │       └── main.rs          # Application entry point
│   └── core
│       ├── Cargo.toml
│       └── src/
│           ├── handlers/
│           │   ├── push.rs           # Handler for push events
│           │   ├── repository.rs     # Handler for repository events
│           │   └── installation.rs   # Handler for app installation events
│           ├── github/
│           │   ├── app.rs            # GitHub App authentication
│           │   ├── repository.rs     # Repository operations
│           │   └── content.rs        # Content operations
│           ├── template/
│           │   ├── config.rs         # Configuration parsing
│           │   └── sync.rs           # Template synchronization logic
│           ├── lib.rs
│           ├── handlers.rs
│           ├── github.rs
│           ├── template.rs
│           └── error.rs              # Error handling
├── terraform/                        # Infrastructure as code
│   ├── aws/                    # AWS-specific terraform
│   │   ├── main.tf             # Main AWS configuration
│   │   ├── variables.tf        # Input variables
│   │   └── outputs.tf          # Output values
│   └── azure/                  # Azure-specific terraform
│       ├── main.tf             # Main Azure configuration
│       ├── variables.tf        # Input variables
│       └── outputs.tf          # Output values
└── docs/                       # Documentation
    ├── setup.md                # Setup instructions
    ├── architecture.md         # Architecture overview
    ├── cli-usage.md            # CLI usage documentation
    └── faq.md                  # Frequently asked questions
```

## Security Considerations

### GitHub App Permissions

The GitHub App should be configured with the minimum required permissions:

1. **Repository Permissions**:
   - **Contents**: Read & Write (to update templates)
   - **Pull Requests**: Read & Write (to create PRs)
   - **Metadata**: Read (to access repo information)
   - **Topics**: Read (to categorize repositories)

2. **Organization Permissions**:
   - **Members**: Read (to validate user permissions)

### Authentication

1. **GitHub App Authentication**:
   - Use the GitHub App JWT-based authentication
   - Rotate private keys regularly (at least every 90 days)
   - Store private keys securely in a key management service (AWS KMS, Azure Key Vault, etc.)

2. **Personal Access Token (for CLI)**:
   - Support for GitHub Personal Access Tokens for CLI authentication
   - Require minimum necessary scopes for the operation
   - Provide guidance on token security best practices

3. **User Authorization**:
   - Respect GitHub's repository-level permissions
   - Verify organization membership for sensitive operations

### Code Security

1. **Code Scanning**:
   - Implement Rust's `cargo audit` for dependency vulnerability scanning
   - Use GitHub's CodeQL for static code analysis
   - Regularly update dependencies

2. **Secrets Management**:
   - Use environment variables for sensitive configuration
   - Never hardcode secrets in the application code
   - Use a secrets management service for production deployments
   - For CLI, support reading credentials from secure credential stores

### Operational Security

1. **Logging**:
   - Log all template update operations
   - Include user/app identity in logs
   - Don't log sensitive information

2. **Rate Limiting**:
   - Implement proper rate limiting for GitHub API calls
   - Use exponential backoff for retries

## Deployment Options

### 1. AWS Lambda Deployment

#### Prerequisites

- AWS Account with appropriate permissions
- AWS CLI configured
- Terraform installed (optional, for IaC deployment)

#### Setup

1. **GitHub App Registration**:
   - Register a new GitHub App in your organization
   - Configure the required permissions
   - Generate and securely store a private key
   - Install the app on repositories that should be managed

2. **Infrastructure Requirements**:
   - AWS Lambda for serverless function
   - API Gateway to receive webhooks
   - AWS Secrets Manager for storing the GitHub App private key
   - CloudWatch for logs and monitoring

#### Deployment

1. **Using Terraform**:
   ```bash
   # Build the Rust Lambda function
   cargo lambda build --release

   # Deploy using Terraform
   cd terraform/aws
   terraform init
   terraform apply
   ```

2. **Manual Deployment**:
   - Build the binary: `cargo lambda build --release`
   - Deploy to Lambda using AWS CLI
   - Configure API Gateway to forward GitHub webhooks to Lambda

3. **Configuration**:
   - Set environment variables in Lambda configuration:
     - `GITHUB_APP_ID`: The GitHub App ID
     - `GITHUB_PRIVATE_KEY`: The GitHub App private key (use AWS Secrets Manager)
     - `MASTER_REPO_OWNER`: The owner of the master template repository
     - `MASTER_REPO_NAME`: The name of the master template repository

### 2. Azure Functions Deployment

#### Prerequisites

- Azure subscription
- Azure CLI installed
- Azure Functions Core Tools installed
- Terraform installed (optional, for IaC deployment)

#### Setup

1. **GitHub App Registration**:
   - Same as for AWS Lambda

2. **Infrastructure Requirements**:
   - Azure Functions for serverless execution
   - Azure Key Vault for storing the GitHub App private key
   - Application Insights for monitoring

#### Deployment

1. **Using Terraform**:
   ```bash
   # Build for Azure Functions
   cargo build --release --target x86_64-unknown-linux-musl

   # Deploy using Terraform
   cd terraform/azure
   terraform init
   terraform apply
   ```

2. **Manual Deployment**:
   ```bash
   # Build for Azure Functions
   cargo build --release --target x86_64-unknown-linux-musl

   # Deploy to Azure
   func azure functionapp publish template-sync-app
   ```

3. **Configuration**:
   - Set application settings in Azure Functions:
     - Same environment variables as AWS

### 3. CLI Application Deployment

#### Prerequisites

- GitHub Personal Access Token or GitHub App credentials
- Rust toolchain installed (for building from source)

#### Installation

1. **From Prebuilt Binaries**:
   ```bash
   # Download the appropriate binary for your platform
   curl -L -o template-sync https://github.com/your-org/template-sync/releases/latest/download/template-sync-$(uname -s)-$(uname -m)
   chmod +x template-sync
   sudo mv template-sync /usr/local/bin/
   ```

2. **From Source**:
   ```bash
   # Clone the repository
   git clone https://github.com/your-org/template-sync.git
   cd template-sync

   # Build the binary
   cargo build --release

   # Install the binary
   sudo cp target/release/template-sync /usr/local/bin/
   ```

#### Usage

```bash
# Configure authentication (one-time setup)
template-sync auth --token YOUR_GITHUB_TOKEN
# or
template-sync auth --app-id APP_ID --key-path /path/to/private-key.pem

# Sync templates for a specific repository
template-sync sync --repo owner/repo

# Sync templates for all repositories in an organization
template-sync sync --org your-organization

# Sync templates based on a local master template
template-sync sync --master-path /path/to/local/templates --repo owner/repo

# Check which repositories need template updates
template-sync check --org your-organization

# View sync history
template-sync history --repo owner/repo
```

## CI/CD Pipeline

Implement a CI/CD pipeline using GitHub Actions:

1. **CI Workflow**:
   - Build and test the application for all deployment targets
   - Run security scans
   - Check code formatting
   - Build release artifacts for CLI distribution

2. **CD Workflow**:
   - Deploy serverless functions to staging environments
   - Run integration tests
   - Deploy to production environments
   - Publish CLI releases

### Monitoring

1. **Application Logs**:
   - Configure log retention across all deployment options
   - Set up alerts for errors

2. **Metrics**:
   - Track template update operations
   - Monitor API rate limits
   - Track function execution times

3. **Health Checks**:
   - Implement a status endpoint for serverless deployments
   - Set up uptime monitoring

## Rollout Strategy

1. **Initial Setup**:
   - Deploy the application to test environments for all deployment options
   - Register and configure the GitHub App
   - Set up the master template repository

2. **Pilot Phase**:
   - Select a small group of repositories for initial testing
   - Test all deployment options with a limited scope
   - Monitor for issues and gather feedback

3. **Full Rollout**:
   - Expand to all repositories
   - Provide documentation for all deployment options
   - Train repository maintainers and DevOps teams

4. **Ongoing Maintenance**:
   - Regularly review and update the application
   - Monitor GitHub API changes
   - Gather feedback from users
   - Maintain compatibility across all deployment options
