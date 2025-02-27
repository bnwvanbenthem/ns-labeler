# tagging-operator
Kubernetes operator to automate the label and annotation logic through a custom resource. 

## Build container
```bash
source ../00-ENV/env.sh
CVERSION="0.1.1"

docker login ghcr.io -u bartvanbenthem -p $CR_PAT

docker build -t tagging-operator:$CVERSION .

docker tag tagging-operator:$CVERSION ghcr.io/bartvanbenthem/tagging-operator:$CVERSION
docker push ghcr.io/bartvanbenthem/tagging-operator:$CVERSION
```

## Deploy Operator
```bash
kubectl apply -f ./config/manager/operator.yaml
```

## CR Spec
```yaml
apiVersion: cncp.nl/v1beta1
kind: Tagger
metadata:
  name: example-tagger
  namespace: default
spec:
  labels:
    - key: "customer"
      value: "my-customer-a"
    - key: "env"
      value: "development"
  annotations:
    - key: "customer"
      value: "my-customer-a"
  excludeList:
    - "kube-system"
    - "monitoring"
```