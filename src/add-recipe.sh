#!/usr/bin/env bash

API_URL="http://localhost:3000/api/v1"

# Get user info
read -p "Full name: " full_name
read -p "Email: " email
read -p "Access code: " access_code
echo

echo "Registering..."
response=$(curl -s -X POST "$API_URL/register" \
    -H "Content-Type: application/json" \
    -d "{\"full_name\": \"$full_name\", \"email\": \"$email\", \"access_code\": \"$access_code\"}"
    )

token=$(echo "$response" | jq -r '.access_token')

if [[ "$token" == "null" ]]; then
    echo
    echo "Registration failed:"
    echo "$response"
    exit 1
fi

echo "Registration complete"
echo "Adding recipe..."

recipe='{
    "id": "water",
    "title": "Water",
    "ingredients": "1 cup water",
    "instructions": "Drink water",
    "tags": ["water"],
    "source": "Jason Gonzales"
}'

curl -X POST -H "Content-type: application/json"  \
     -H "Authorization: Bearer $token" \
     -d "$recipe" http://localhost:3000/api/v1/add-recipe

echo "Recipe added."
