SPOTS = {}
SPOTS["lagouvardos"] = dict(
    name="Lagouvardos",
    region="Peloponnese",          # Messinia, west coast

    # Location (beach itself; wave_point() snaps to the nearest sea cell)
    lat=37.09,
    lon=21.58,

    # Geometry, all "coming from" degrees
    facing=260,                    # beach looks WSW/W (map shows a N-S coast, sea to the west)
    window=60,                     # swell window 200-320, covers SSW through NW
    ideal_swell_dir=(235, 285),    # WSW to W is the sweet spot

    # Size and period (starting guesses, tune from real sessions)
    min_hs=1.0,                    # below this, "not surfable"
    max_hs=3.0,                    # above this, closes out / too much
    min_period=4.0,                # short-fetch Med chop floor
    ideal_period=(6.0, 9.0),

    min_power_kw_m=5.0,      # below this: too weak
    ideal_power_kw_m=15.0,    # full energy score at or above this

    # Wind, relative to facing 260 (offshore blows from the east)
    offshore_dir=(40, 130),        # clean
    max_onshore_kn=8,              # beyond this, wind waves dominate and it gets messy
    max_offshore_kn=20,            # beyond this, it blows the face out flat

    # Type
    break_type="beach",
    notes="Exposed beach break, works occasionally. Best on W/WSW swell with E wind.",
)

SPOTS["krokala"] = dict(
    name="Krokala",
    region="Peloponnese",          # Messinia, southwest coast near Foinikounda

    # Location (ESTIMATE from the map labels, TODO: drop real pin in)
    lat=36.83,
    lon=21.79,

    # Geometry, all "coming from" degrees
    facing=265,                    # beach looks W (the cone points due west, swell arrow W 277)
    window=55,                     # swell window 210-320, open to SW through NW
    ideal_swell_dir=(240, 290),    # WSW to WNW

    # Size and period (starting guesses, tune from real sessions)
    min_hs=1.5,                    # small bay, but the model Hs is offshore so keep a floor
    max_hs=4.5,
    min_period=4.0,
    ideal_period=(6.0, 9.0),

    min_power_kw_m=5.0,      # below this: too weak
    ideal_power_kw_m=15.0,    # full energy score at or above this

    # Wind, relative to facing 265 (offshore blows from the east)
    offshore_dir=(45, 135),
    max_onshore_kn=8, 
    max_offshore_kn=20,

    # Type
    break_type="beach",
    notes="Open to the W. Works on W/WSW swell with E wind, like Lagouvardos but a bit more sheltered.",
)

SPOTS["vouliagmeni"] = dict(
    name="Vouliagmeni",
    region="Attica",               # Saronic Gulf, Athens Riviera

    # Location (approximate, confirm against your pin; wave_point() snaps to nearest sea cell)
    lat=37.81,
    lon=23.78,

    # Geometry, all "coming from" degrees
    facing=180,                    # bay mouth opens due south (map shows the cone pointing S)
    window=40,                     # swell window 140-220, narrow because the bay is sheltered
    ideal_swell_dir=(165, 205),    # S to SSW

    # Size and period (starting guesses, tune from real sessions)
    min_hs=1.0,                    # sheltered bay, so smaller waves count
    max_hs=2.5,
    min_period=4.0,                # Saronic events are short-period wind waves
    ideal_period=(5.0, 7.0),

    min_power_kw_m=5.0,      # below this: too weak
    ideal_power_kw_m=15.0,    # full energy score at or above this

    # Wind, relative to facing 180 (offshore blows from the north)
    offshore_dir=(320, 40),        # wraps through 0, so use angdiff(), not a plain range check
    max_onshore_kn=15,             # south wind makes the waves here, but too much turns it to mush
    max_offshore_kn=20,

    # Swell source
    use_wind_waves=True,           # judge on total Hs / VHM0_WW, not only the SW1 swell partition

    break_type="beach/bay",
    notes="Saronic Gulf: only works on S/SSW wind events (notos). Short period, wind dependent.",
)

SPOTS["messakti"] = dict(
    name="Messakti",
    region="Ikaria",

    # Location (approximate, confirm against your pin; wave_point() snaps to nearest sea cell)
    lat=37.63,
    lon=26.03,

    # Geometry, all "coming from" degrees
    facing=350,                      # bay opens north (cone points NNW in the map)
    window=50,                       # swell window 300-40, wraps through 0
    ideal_swell_dir=(340, 30),       # NNW to NNE, meltemi sector

    # Size and period (starting guesses, tune from real sessions)
    min_hs=1.0,
    max_hs=3.0,
    min_period=4.0,                  # meltemi wind sea is short and steep
    ideal_period=(5.0, 8.0),

    min_power_kw_m=5.0,      # below this: too weak
    ideal_power_kw_m=15.0,    # full energy score at or above this

    # Wind, relative to facing 350 (offshore blows from the south)
    offshore_dir=(140, 220),
    max_onshore_kn=30,               # meltemi IS onshore here, so the cap is high
    max_offshore_kn=20,

    # Meltemi driver
    use_wind_waves=True,             # judge on total Hs / VHM0_WW, not the swell partition
    driver_wind_dir=(340, 50),       # N to NE
    min_driver_wind_kn=15,           # needs a real meltemi, not just a breeze
    season_months=(6, 9),            # Jun-Sep; peaks Jul-Aug

    break_type="beach/bay",
    notes="Works almost only on summer meltemi wind-sea. Wind and waves arrive together.",
)

# TODO: liani ammos, Davis, tinos, Crete (chrisi akti, falarsana)



# TODO: manual run to update spots in firestore/where ever