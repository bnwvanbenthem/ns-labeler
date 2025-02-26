kubectl apply -f config/crd/taggers.cndev.nl.yaml

kubectl apply -f config/samples/tagger-example.yaml

cargo test
cargo run

# kubectl delete -f config/samples/tagger-example.yaml

# kubectl delete -f config/crd/taggers.cndev.nl.yaml