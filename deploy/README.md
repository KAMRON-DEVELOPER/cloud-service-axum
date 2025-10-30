# Deployment Guide

This guide covers deploying PostgreSQL, Redis, RabbitMQ, and Kafka services to your K3s cluster.

## Prerequisites

- K3s cluster running (master + agent nodes)
- `kubectl` configured to connect to your cluster
- Local storage provisioner enabled (k3s includes `local-path` by default)

## Setup Steps

### 1. Create Configuration ConfigMaps

These ConfigMaps contain service configuration files that will be mounted into the pods.

#### PostgreSQL Configuration

```bash
kubectl create configmap postgres-conf -n postgres-ns \
  --from-file=postgresql.conf=deploy/configurations/postgresql/postgresql.local.conf \
  --from-file=pg_hba.conf=deploy/configurations/postgresql/pg_hba.local.conf
```

#### Redis Configuration

```bash
kubectl create configmap redis-conf -n redis-ns \
  --from-file=redis.conf=deploy/configurations/redis-stack/redis-stack.local.conf
```

#### RabbitMQ Configuration

```bash
kubectl create configmap rabbitmq-conf -n rabbitmq-ns \
  --from-file=rabbitmq.conf=deploy/configurations/rabbitmq/rabbitmq.local.conf
```

### 2. Deploy Services

Apply all deployment manifests:

```bash
kubectl apply -f deploy/postgres.yaml
kubectl apply -f deploy/redis.yaml
kubectl apply -f deploy/rabbitmq.yaml
kubectl apply -f deploy/kafka.yaml
```

Or apply all at once:

```bash
kubectl apply -f deploy/
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

## Service Details

### PostgreSQL

- **Namespace**: `postgres-ns`
- **Service**: `postgres-service` (ClusterIP)
- **Headless Service**: `postgres-hs`
- **Image**: `postgres:bookworm`
- **User**: `postgres`
- **Password**: `password` (from Secret)
- **Database**: `cloud_service_db`
- **Storage**: 1Gi PVC per pod

### Redis

- **Namespace**: `redis-ns`
- **Service**: `redis-service` (ClusterIP)
- **Headless Service**: `redis-hs`
- **Image**: `redis/redis-stack-server:latest`
- **User**: `default`
- **Password**: `password` (from Secret)
- **Storage**: 1Gi PVC per pod
- **Features**: RediSearch, ReJSON modules enabled

### RabbitMQ

- **Namespace**: `rabbitmq-ns`
- **Service**: `rabbitmq-service` (ClusterIP)
- **Headless Service**: `rabbitmq-hs`
- **Image**: `rabbitmq:management-alpine`
- **User**: `guest`
- **Password**: `password` (from Secret)
- **Ports**: 5672 (AMQP), 15672 (Management UI)
- **Storage**: 1Gi PVC per pod

### Kafka

- **Namespace**: `kafka-ns`
- **Headless Service**: `kafka-hs`
- **Image**: `apache/kafka:latest`
- **Mode**: KRaft (no ZooKeeper)
- **Replicas**: 3 brokers
- **Replication Factor**: 3
- **Min ISR**: 2
- **Storage**: 1Gi PVC per pod

## Cleanup

To remove all services:

```bash
kubectl delete -f deploy/postgres.yaml
kubectl delete -f deploy/redis.yaml
kubectl delete -f deploy/rabbitmq.yaml
kubectl delete -f deploy/kafka.yaml
```

Delete ConfigMaps:

```bash
kubectl delete configmap postgres-conf -n postgres-ns
kubectl delete configmap redis-conf -n redis-ns
kubectl delete configmap rabbitmq-conf -n rabbitmq-ns
```

## Troubleshooting

### View logs

```bash
kubectl logs -n <namespace> <pod-name>
```

### Describe resources

```bash
kubectl describe pod -n <namespace> <pod-name>
kubectl describe statefulset -n <namespace> <statefulset-name>
```

### Execute commands in pods

```bash
kubectl exec -it -n <namespace> <pod-name> -- /bin/bash
```

### Check persistent volumes

```bash
kubectl get pv
kubectl get pvc --all-namespaces
```
