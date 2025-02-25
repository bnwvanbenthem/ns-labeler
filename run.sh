kubectl apply -f config/crd/labelers.cndev.nl.yaml

kubectl apply -f config/samples/labeler-example.yaml

cargo test
cargo run

# kubectl delete -f config/samples/labeler-example.yaml

# kubectl delete -f config/crd/labelers.cndev.nl.yaml