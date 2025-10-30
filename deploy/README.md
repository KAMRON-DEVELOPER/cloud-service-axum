# Deployment Guide

This guide covers deploying PostgreSQL, Redis, RabbitMQ, and Kafka services to your K3s cluster.

## Prerequisites

- K3s cluster running (master + agent nodes)
- `kubectl` configured to connect to your cluster
- Local storage provisioner enabled (k3s includes `local-path` by default)

## Setup Steps

### 1. Create Configuration ConfigMaps

These ConfigMaps contain service configuration files that will be mounted into the pods. **Note**: The configuration files are created from the local `.conf` files using `kubectl create configmap --from-file`.

#### PostgreSQL Configuration

```bash
kubectl create namespace postgres-ns
kubectl create configmap postgres-conf -n postgres-ns \
  --from-file=postgresql.conf=configurations/postgresql/postgresql.local.conf \
  --from-file=pg_hba.conf=configurations/postgresql/pg_hba.local.conf
```

#### Redis Configuration

```bash
kubectl create namespace redis-ns
kubectl create configmap redis-conf -n redis-ns \
  --from-file=redis.conf=configurations/redis-stack/redis-stack.local.conf
```

#### RabbitMQ Configuration

```bash
kubectl create namespace rabbitmq-ns
kubectl create configmap rabbitmq-conf -n rabbitmq-ns \
  --from-file=rabbitmq.conf=configurations/rabbitmq/rabbitmq.local.conf
```

#### Kafka Configuration

Kafka uses a ConfigMap defined in `kafka.yaml` for the cluster ID. No separate ConfigMap creation needed.

### 2. Deploy Services

Apply all deployment manifests:

```bash
kubectl apply -f postgres.yaml
kubectl apply -f redis.yaml
kubectl apply -f rabbitmq.yaml
kubectl apply -f kafka.yaml
```

Or apply all at once:

```bash
kubectl apply -f .
```

### 3. Verify Deployments

Check that all pods are running:

```bash
kubectl get pods -n postgres-ns
kubectl get pods -n redis-ns
kubectl get pods -n rabbitmq-ns
kubectl get pods -n kafka-ns
```

Check StatefulSets:

```bash
kubectl get statefulsets --all-namespaces
```

Check Services:

```bash
kubectl get svc --all-namespaces | grep -E "postgres|redis|rabbitmq|kafka"
```

## Port Forwarding for Local Access

To access services from your local machine, use port forwarding:

### PostgreSQL

```bash
kubectl port-forward -n postgres-ns svc/postgres-service 5432:5432
```

Connection string: `postgresql://postgres:password@localhost:5432/cloud_service_db`

### Redis

```bash
kubectl port-forward -n redis-ns svc/redis-service 6379:6379
```

Connection string: `redis://default:password@localhost:6379`

### RabbitMQ

```bash
kubectl port-forward -n rabbitmq-ns svc/rabbitmq-service 5672:5672 15672:15672
```

- AMQP: `amqp://guest:password@localhost:5672`
- Management UI: `http://localhost:15672` (username: `guest`, password: `password`)

### Kafka

```bash
# Port-forward to a specific broker pod
kubectl port-forward -n kafka-ns kafka-ss-0 9092:9092
```

Bootstrap server: `localhost:9092`

**Note**: For external access to all Kafka brokers, you would need to port-forward to each broker separately or configure proper external listeners.

## Service Details

### PostgreSQL

- **Namespace**: `postgres-ns`
- **Service**: `postgres-service` (ClusterIP on port 5432)
- **Headless Service**: `postgres-hs`
- **StatefulSet**: `postgres-ss` (1 replica)
- **Image**: `postgres:bookworm`
- **Configuration**: Custom `postgresql.conf` and `pg_hba.conf` from ConfigMap
- **User**: `postgres` (from ConfigMap)
- **Password**: `password` (from Secret)
- **Database**: `cloud_service_db` (from ConfigMap)
- **Storage**: 1Gi PVC per pod (`local-path` storage class)

**Custom Config Features**:

- Listens on all interfaces (`listen_addresses = '*'`)
- SCRAM-SHA-256 authentication
- Max 100 connections
- 128MB shared buffers

### Redis

- **Namespace**: `redis-ns`
- **Service**: `redis-service` (ClusterIP on port 6379)
- **Headless Service**: `redis-hs`
- **StatefulSet**: `redis-ss` (1 replica)
- **Image**: `redis/redis-stack-server:latest`
- **Configuration**: Custom `redis.conf` from ConfigMap
- **User**: `default` (from ConfigMap)
- **Password**: `password` (from Secret)
- **Storage**: 1Gi PVC per pod (`local-path` storage class)
- **Resources**:
  - Requests: 200m CPU, 256Mi memory
  - Limits: 1 CPU, 1Gi memory

**Custom Config Features**:

- AOF persistence enabled
- 4GB max memory with LRU eviction
- RediSearch and ReJSON modules loaded
- Automatic snapshots (3600s/1 change, 300s/100 changes, 60s/10000 changes)
- RDB compression enabled

**Health Checks**:

- Readiness probe: `redis-cli ping` every 10s
- Liveness probe: `redis-cli ping` every 20s

### RabbitMQ

- **Namespace**: `rabbitmq-ns`
- **Service**: `rabbitmq-service` (ClusterIP)
  - AMQP: port 5672
  - Management UI: port 15672
- **Headless Service**: `rabbitmq-hs`
- **StatefulSet**: `rabbitmq-ss` (1 replica, parallel pod management)
- **Image**: `rabbitmq:management-alpine`
- **Configuration**: Custom `rabbitmq.conf` from ConfigMap
- **User**: `guest` (from ConfigMap)
- **Password**: `password` (from Secret)
- **Storage**: 1Gi PVC per pod (`local-path` storage class)
- **Resources**:
  - Requests: 200m CPU, 256Mi memory
  - Limits: 1 CPU, 1Gi memory

**Custom Config Features**:

- TCP listener on port 5672
- SHA-512 password hashing

### Kafka

- **Namespace**: `kafka-ns`
- **Headless Service**: `kafka-hs` (with `publishNotReadyAddresses: true`)
  - Kafka: port 9092
  - Controller: port 9093
- **StatefulSet**: `kafka-ss` (3 replicas, parallel pod management)
- **Image**: `apache/kafka:latest`
- **Mode**: KRaft (no ZooKeeper required)
- **Cluster ID**: `MkU3OEVBNTcwNTJENDM2Qk` (from ConfigMap)
- **Storage**: 1Gi PVC per pod (`local-path` storage class)

**Cluster Configuration**:

- 3 brokers in KRaft mode (each pod is both broker and controller)
- Replication factor: 3
- Transaction state log replication: 3
- Min ISR: 2
- Quorum voters: All 3 brokers participate in controller quorum

**Pod Configuration**:

- Each pod gets a unique node ID from its hostname (0, 1, or 2)
- Advertised listeners use pod's DNS name within the cluster
- Storage formatted automatically on first start
- Listeners: PLAINTEXT on 9092, CONTROLLER on 9093

**DNS Names for Brokers**:

- `kafka-ss-0.kafka-hs.kafka-ns.svc.cluster.local:9092`
- `kafka-ss-1.kafka-hs.kafka-ns.svc.cluster.local:9092`
- `kafka-ss-2.kafka-hs.kafka-ns.svc.cluster.local:9092`

## Updating Configurations

If you need to update configuration files after deployment:

### PostgreSQL

```bash
kubectl delete configmap postgres-conf -n postgres-ns
kubectl create configmap postgres-conf -n postgres-ns \
  --from-file=postgresql.conf=configurations/postgresql/postgresql.local.conf \
  --from-file=pg_hba.conf=configurations/postgresql/pg_hba.local.conf
kubectl rollout restart statefulset/postgres-ss -n postgres-ns
```

### Redis

```bash
kubectl delete configmap redis-conf -n redis-ns
kubectl create configmap redis-conf -n redis-ns \
  --from-file=redis.conf=configurations/redis-stack/redis-stack.local.conf
kubectl rollout restart statefulset/redis-ss -n redis-ns
```

### RabbitMQ

```bash
kubectl delete configmap rabbitmq-conf -n rabbitmq-ns
kubectl create configmap rabbitmq-conf -n rabbitmq-ns \
  --from-file=rabbitmq.conf=configurations/rabbitmq/rabbitmq.local.conf
kubectl rollout restart statefulset/rabbitmq-ss -n rabbitmq-ns
```

### Kafka

Kafka's ConfigMap is defined in `kafka.yaml`. To update environment variables, edit `kafka.yaml` and reapply:

```bash
kubectl apply -f kafka.yaml
kubectl rollout restart statefulset/kafka-ss -n kafka-ns
```

## Cleanup

To remove all services:

```bash
kubectl delete -f postgres.yaml
kubectl delete -f redis.yaml
kubectl delete -f rabbitmq.yaml
kubectl delete -f kafka.yaml
```

Delete ConfigMaps:

```bash
kubectl delete configmap postgres-conf -n postgres-ns
kubectl delete configmap redis-conf -n redis-ns
kubectl delete configmap rabbitmq-conf -n rabbitmq-ns
```

Delete namespaces (this will remove all resources):

```bash
kubectl delete namespace postgres-ns
kubectl delete namespace redis-ns
kubectl delete namespace rabbitmq-ns
kubectl delete namespace kafka-ns
```

## Troubleshooting

### View logs

```bash
kubectl logs -n <namespace> <pod-name>
# Follow logs
kubectl logs -n <namespace> <pod-name> -f
# View previous container logs
kubectl logs -n <namespace> <pod-name> --previous
```

### Describe resources

```bash
kubectl describe pod -n <namespace> <pod-name>
kubectl describe statefulset -n <namespace> <statefulset-name>
kubectl describe svc -n <namespace> <service-name>
```

### Execute commands in pods

```bash
# PostgreSQL
kubectl exec -it -n postgres-ns postgres-ss-0 -- psql -U postgres -d cloud_service_db

# Redis
kubectl exec -it -n redis-ns redis-ss-0 -- redis-cli

# RabbitMQ
kubectl exec -it -n rabbitmq-ns rabbitmq-ss-0 -- rabbitmqctl status

# Kafka
kubectl exec -it -n kafka-ns kafka-ss-0 -- /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
```

### Check persistent volumes

```bash
kubectl get pv
kubectl get pvc --all-namespaces
```

### Kafka-specific troubleshooting

Check cluster status:

```bash
kubectl exec -it -n kafka-ns kafka-ss-0 -- \
  /opt/kafka/bin/kafka-metadata.sh --snapshot /var/lib/kafka/data/__cluster_metadata-0/00000000000000000000.log --print-records
```

List topics:

```bash
kubectl exec -it -n kafka-ns kafka-ss-0 -- \
  /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 --list
```

Create a test topic:

```bash
kubectl exec -it -n kafka-ns kafka-ss-0 -- \
  /opt/kafka/bin/kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic test-topic --partitions 3 --replication-factor 3
```

### Common Issues

**Pods not starting**: Check events with `kubectl describe pod` and review logs.

**Storage issues**: Verify PVCs are bound with `kubectl get pvc -n <namespace>`.

**ConfigMap not found**: Ensure you've created the ConfigMaps before applying the deployment manifests.

**Kafka brokers not forming quorum**: Check that all 3 pods are running and can communicate via the headless service.

## Production Considerations

For production deployments, consider:

1. **Security**: Change default passwords in Secrets
2. **Storage**: Use appropriate storage classes with backup capabilities
3. **Resources**: Adjust CPU and memory limits based on workload
4. **High Availability**: Increase replicas for PostgreSQL, Redis, and RabbitMQ
5. **Monitoring**: Add Prometheus exporters and monitoring
6. **Networking**: Consider NetworkPolicies for pod-to-pod communication
7. **Backups**: Implement automated backup strategies for stateful services
8. **TLS**: Enable TLS for all service connections
