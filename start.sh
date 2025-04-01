#!/bin/bash

echo "Starting Supabase..."
cd supabase
docker compose up -d

echo "Waiting for Supabase to be healthy..."
until curl -s http://localhost:54321/rest/v1/ > /dev/null; do
    echo "Waiting for Supabase..."
    sleep 5
done

echo "Supabase is ready! Starting main services..."
cd ..
docker compose up 