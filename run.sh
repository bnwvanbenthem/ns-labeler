kubectl apply -f config/crd/taggers.cndev.nl.yaml

cargo test
cargo run


kubectl apply -f config/samples/tagger-example.yaml
kubectl get ns --show-labels
kubectl describe taggers.cncp.nl example-tagger

kubectl apply -f config/samples/tagger-example-test-exclude.yaml
kubectl get ns --show-labels
kubectl describe taggers.cncp.nl example-tagger

# kubectl delete -f config/samples/tagger-example.yaml

# kubectl delete -f config/crd/taggers.cndev.nl.yaml