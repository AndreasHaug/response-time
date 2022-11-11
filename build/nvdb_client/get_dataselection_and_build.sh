
rm -fr .venv

python -m venv .venv
source .venv/bin/activate

pip install -r requirements.txt
# python3 get_data_selection.py
cd build_db/
# ./load_raw.sh
# ./extract_speedlimits_from_raw.sh
./load_from_raw.sh

deactivate

