# Kubernetes Cluster with Proxy Deployment

This document explains how to create a Kubernetes cluster using Kind, apply DNS rewrites using CoreDNS, and deploy a proxy.

## Prerequisites

- Docker
- Kind
- kubectl

## Steps

### 1. Create the Cluster

Run the following command to create the Kubernetes cluster:

```bash
./cluster-create
```

This script will spin up a Kind Kubernetes cluster using the configuration file `k8s/kind.yaml`.

### 2. Modify CoreDNS Configuration

After creating the cluster, update CoreDNS for DNS rewriting.

To manually edit this configuration, you can run:

```bash
kubectl -n kube-system edit configmap coredns
```

Insert the rewrite plugin line as shown above:

```yaml
rewrite name exact www.google.com server.default.svc.cluster.local
```

After updating CoreDNS, restart the deployment:

```bash
kubectl -n kube-system rollout restart deployment coredns
```

#### CoreDNS example after rewriring

```yaml
apiVersion: v1
data:
  Corefile: |
    .:53 {
        errors
        health {
           lameduck 5s
        }
        ready

        rewrite name exact www.google.com server.default.svc.cluster.local

        kubernetes cluster.local in-addr.arpa ip6.arpa {
           pods insecure
           fallthrough in-addr.arpa ip6.arpa
           ttl 30
        }
        prometheus :9153
        forward . /etc/resolv.conf {
           max_concurrent 1000
        }
        cache 30
        loop
        reload
        loadbalance
    }
kind: ConfigMap
metadata:
  name: coredns
  namespace: kube-system
```

### 3. Deploy the Proxy

Once the CoreDNS has been configured, run the following command to deploy the proxy:

```bash
./proxy-deploy.sh
```

This script will apply the `k8s/server.yaml` and `k8s/requests.yaml` configurations to the Kubernetes cluster, setting up the necessary pods.

---

## Summary

The following steps summarize the process:

1. **Create Cluster**:

   ```bash
   ./cluster-create.sh
   ```

2. **Edit CoreDNS**:

   ```bash
   kubectl -n kube-system edit configmap coredns
   ```

   Insert the rewrite rule:

   ```yaml
   rewrite name exact www.google.com server.default.svc.cluster.local
   ```

3. **Restart CoreDNS**:

   ```bash
   kubectl -n kube-system rollout restart deployment coredns
   ```

4. **Deploy Proxy**:

   ```bash
   ./proxy-deploy.sh
   ```

5. **Check Proxy Logs**
   ```bash
   kubectl logs -f server
   ```
