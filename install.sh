#!/bin/bash

echo "Creating the python env in .passdb"
python3 -m venv .passdb

echo "Mounting into the python environnement and installing the requirements"
source .passdb/bin/activate
python3 -m pip install -r requirements.txt
