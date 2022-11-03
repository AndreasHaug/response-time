!# bin/bash

python -m venv .venv
source .venv/bin/activate


python3 get_data_selection.py
cd build_db/
./load_raw.sh
./extract_speedlimits_from_raw.py
./load_from_raw.py

deactivate

