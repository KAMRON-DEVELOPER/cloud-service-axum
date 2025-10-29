# Deployments

## 1. Create the Postgres ConfigMap from your files

`kubectl create configmap postgres-cm -n postgres-ns \
  --from-file=deploy/configurations/postgresql/postgresql.conf \
  --from-file=deploy/configurations/postgresql/pg_hba.conf \
  --from-file=deploy/configurations/postgresql/postgres-entrypoint.sh`

## 2. Create the Redis ConfigMap

`kubectl create configmap redis-config -n redis-ns \
  --from-file=redis.conf=deploy/configurations/redis-stack/redis-stack.local.conf`

## 3. Create the RabbitMQ ConfigMap

`kubectl create configmap rabbitmq-config -n rabbitmq-ns \
  --from-file=rabbitmq.conf=deploy/configurations/rabbitmq/rabbitmq.local.conf`

## For Postgres

`kubectl create configmap postgres-cm -n postgres-ns \
  --from-file=postgres-entrypoint.sh=deploy/configurations/postgresql/postgres-entrypoint.sh \
  --from-file=postgresql.conf=deploy/configurations/postgresql/postgresql.conf \
  --from-file=pg_hba.conf=deploy/configurations/postgresql/pg_hba.conf`

## For Redis (assuming local.conf is what you want)

`kubectl create configmap redis-config -n redis-ns \
  --from-file=redis.conf=deploy/configurations/redis-stack/redis-stack.local.conf`

## For RabbitMQ (assuming local.conf is what you want)

`kubectl create configmap rabbitmq-config -n rabbitmq-ns \
  --from-file=rabbitmq.conf=deploy/configurations/rabbitmq/rabbitmq.local.conf`

## For Kafka (Kafka seems to be missing its ConfigMap YAML, but your StatefulSet references `kafka-cm`)

## You'll need to create `kafka-cm` with a CLUSTER_ID file, e.g

## echo $(/opt/kafka/bin/kafka-storage.sh random-uuid) > /tmp/CLUSTER_ID

## kubectl create configmap kafka-cm -n kafka-ns --from-file=CLUSTER_ID=/tmp/CLUSTER_ID

`kubectl apply -f deploy/`

## Terminal 1: Postgres

`kubectl port-forward -n postgres-ns svc/postgres-service 5432:5432`

## Terminal 2: Redis

`kubectl port-forward -n redis-ns svc/redis-service 6379:6379`

## Terminal 3: RabbitMQ

`kubectl port-forward -n rabbitmq-ns svc/rabbitmq-service 5672:5672 15672:15672`

## Terminal 4: Kafka (must target a specific pod, as the service is headless)

`kubectl port-forward -n kafka-ns kafka-ss-0 9092:9092`
