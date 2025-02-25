# ns-labeler
ns labeler

## CR Spec
```yaml
apiVersion: cndev.nl/v1beta1
kind: Labeler
metadata:
  name: example-labeler
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
  exclude_list:
    - "kube-system"
    - "monitoring"
```

## CRD Spec
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: labelers.cndev.nl
spec:
  group: cndev.nl
  names:
    kind: Labeler
    plural: labelers
  scope: Namespaced
  versions:
    - name: v1beta1
      schema:
        openAPIV3Schema:
          type: object
          properties:
            metadata:
              type: object
            spec:
              type: object
              properties:
                labels:
                  type: array
                  items:
                    type: object
                    properties:
                      key:
                        type: string
                      value:
                        type: string
                annotations:
                  type: array
                  items:
                    type: object
                    properties:
                      key:
                        type: string
                      value:
                        type: string
                exclude_list:
                  type: array
                  items:
                    type: string
            status:
              type: object
              properties:
                succeeded:
                  type: boolean
          required:
            - spec
      served: true
      storage: true
      subresources:
        # status enables the status subresource.
        status: {}
```
