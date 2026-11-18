# NextWave
software to generate events based on surfable locations in the mediterranean sea

# Mechanics
GitHub Actions runs coprn.py
it downloads data from Copernicus
sends .json to Rust engine
Rust engine computes and spits out .json
python writes it to firestore





# Data Dict
Data	Used for
VHM0, VTM10	Power (energy factor and gate), steepness
VTPK	Period gate and factor
VMDR, VMDR_SW1	Swell window gate, direction factor
VHM0_SW1, VTM01_SW1	Swell power at groundswell spots, cleanliness
VHM0_SW2 etc.	Added to swell energy
Wind-wave partition (_WW)	Cross-sea factor
Wind speed and direction (3 models)	Consensus for gate and factor, spread for confidence
Gusts	Gustiness penalty
Open-Meteo marine waves	Second opinion for confidence
weather_code, rain	Thunderstorm gate, small rain penalty
Air temp, sea temp, pressure	AI summary facts only
Time, lat, lon	Season, daylight, lead time
Cell distance	Reliability gate and confidence