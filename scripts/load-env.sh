#!/bin/sh
export GITHUB_TOKEN=$(grep GITHUB_TOKEN .env | cut -d= -f2)
export CONTEXT7_API_KEY=$(grep CONTEXT7_API_KEY .env | cut -d= -f2)
echo "Environment variables loaded for opencode"
