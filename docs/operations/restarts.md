# Restart Behavior

This project does not restart workloads by itself. The webhook resolves values;
External Secrets Operator writes Kubernetes Secrets; restart behavior should be
chosen explicitly per workload.

Recommended options:

- Use applications that reload mounted Secret files or environment-derived
  config themselves when possible.
- Use Stakater Reloader for annotation-driven Deployment restarts after Secret
  changes.
- Use a chart checksum annotation when the Secret is rendered by the same Helm
  release as the workload.
- Use GitOps-controlled force-sync annotations on `ExternalSecret` resources
  for manual refreshes.

The live smoke script verifies that after the webhook Deployment restarts, ESO
can force-refresh and keep the target Secret valid. That proves provider
statelessness, but it does not imply every consuming workload will observe the
new Secret without its own reload or restart mechanism.
