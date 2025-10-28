# 1. Create the Postgres ConfigMap from your files

kubectl create configmap postgres-cm -n postgres-ns \
  --from-file=deploy/configurations/postgresql/postgresql.conf \
  --from-file=deploy/configurations/postgresql/pg_hba.conf \
  --from-file=deploy/configurations/postgresql/postgres-entrypoint.sh

# 2. Create the Redis ConfigMap

kubectl create configmap redis-config -n redis-ns \
  --from-file=redis.conf=deploy/configurations/redis-stack/redis-stack.local.conf

# 3. Create the RabbitMQ ConfigMap

kubectl create configmap rabbitmq-config -n rabbitmq-ns \
  --from-file=rabbitmq.conf=deploy/configurations/rabbitmq/rabbitmq.local.conf
