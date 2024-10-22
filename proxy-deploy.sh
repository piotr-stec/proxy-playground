#!/usr/bin/env bash

kubectl apply -f k8s/server.yaml

kubectl apply -f k8s/requests.yaml