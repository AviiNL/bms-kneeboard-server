use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize)]
pub struct Overview<'a> {
    pub callsign: &'a str,
    pub mission_type: &'a str,
    pub package_id: i32,
    pub package_description: &'a str,
    pub package_mission: &'a str,
    pub target_area: &'a str,
    pub time_on_target: &'a str,
    pub sunrise: &'a str,
    pub sunset: &'a str,
}

impl<'a> Overview<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let Some(overview) = extract_group(briefing, "Mission Overview") else {
            return Default::default();
        };

        let lines: Vec<&str> = overview.lines().skip(2).map(str::trim).collect();

        let callsign = lines.first().unwrap_or(&"").split(' ').nth(0).unwrap_or("");
        let mission_type = lines
            .first()
            .unwrap_or(&"")
            .split(' ')
            .nth(1)
            .map(|s| s.trim_matches(&['(', ')']))
            .unwrap_or("");

        let package_id: i32 = lines[2]
            .split(':')
            .nth(1) // Take the second part after splitting by ':'
            .and_then(|s| s.split_whitespace().next()) // Take the first word after trimming whitespace
            .and_then(|s| s.parse().ok()) // Parse the first word into an i32
            .unwrap_or(0); // Unwrap the result

        let package_description = lines[2]
            .split(':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .split('(')
            .nth(1)
            .map(|s| s.trim_matches(|c| c == ')'))
            .unwrap_or("");

        let package_mission = lines[3].split(':').nth(1).unwrap_or("").trim();
        let target_area = lines[4].split(':').nth(1).unwrap_or("").trim();
        let time_on_target = lines[5].split_once(':').unwrap_or(("", "")).1.trim();

        let sunrise = lines[7].split_once(':').unwrap_or(("", "")).1.trim();
        let sunset = lines[8].split_once(':').unwrap_or(("", "")).1.trim();

        Self {
            callsign,
            mission_type,
            package_id,
            package_description,
            package_mission,
            target_area,
            time_on_target,
            sunrise,
            sunset,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Sitrep(pub String);

impl Sitrep {
    pub fn from_briefing(briefing: &str) -> Self {
        let Some(sitrep) = extract_group(briefing, "Situation") else {
            return Default::default();
        };

        let lines: Vec<&str> = sitrep
            .lines()
            .skip(2)
            .map(str::trim)
            .filter(|&l| !l.is_empty())
            .collect();

        let sitrep = lines.join("\n");

        Self(sitrep)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PilotRoster<'a> {
    pub primary: bool,
    pub callsign: &'a str,
    pub lead: &'a str,
    pub wing: &'a str,
    pub element: &'a str,
    pub four: &'a str,
}

impl<'a> PilotRoster<'a> {
    pub fn from_briefing(briefing: &'a str) -> Vec<Self> {
        let Some(overview) = extract_group(briefing, "Mission Overview") else {
            return Default::default();
        };

        let lines: Vec<&str> = overview.lines().skip(2).collect();
        let own = lines
            .first()
            .unwrap_or(&"")
            .split(' ')
            .nth(0)
            .unwrap_or("")
            .trim();

        let Some(data) = extract_group(briefing, "Pilot Roster") else {
            return Default::default();
        };

        let mut roster = vec![];
        let lines: Vec<&str> = data.lines().skip(4).collect();
        for line in lines {
            let mut parts = line.split('\t').map(str::trim).filter(|l| !l.is_empty());
            let callsign = parts.next().unwrap_or("");

            let primary = callsign.eq(own);

            let lead = parts.next().unwrap_or("N/A");
            let wing = parts.next().unwrap_or("N/A");
            let element = parts.next().unwrap_or("N/A");
            let four = parts.next().unwrap_or("N/A");
            let r = Self {
                primary,
                callsign,
                lead,
                wing,
                element,
                four,
            };
            roster.push(r);
        }

        roster
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageElement {
    pub is_primary: bool,
    pub callsign: String,
    pub flight: String,
    pub role: String,
    pub aircraft: String,
    pub task: String,
}

impl PackageElement {
    pub fn from_briefing(briefing: &str) -> Vec<Self> {
        let Some(overview) = extract_group(briefing, "Mission Overview") else {
            return Default::default();
        };

        let lines: Vec<&str> = overview.lines().skip(2).collect();
        let own = lines
            .first()
            .unwrap_or(&"")
            .split(' ')
            .nth(0)
            .unwrap_or("")
            .trim();

        let Some(elements) = extract_group(briefing, "Package Elements") else {
            return Default::default();
        };

        let lines: Vec<&str> = elements
            .lines()
            .skip(4)
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        let mut flights = vec![];
        for f in (0..lines.len() - 1).step_by(2) {
            let lines: Vec<&str> = vec![lines[f], lines[f + 1]];

            let line1: Vec<&str> = lines.first().unwrap_or(&"").split('\t').collect();
            let line2: Vec<&str> = lines[1].split('\t').collect();

            let is_primary = line1.first().unwrap_or(&"").contains(own);

            let callsign = line1.first().unwrap_or(&"").to_string();
            let flight = format!("{}\n{}", line1[1], line2.first().unwrap_or(&""));
            let role = format!("{}\n{}", line1[2], line2[1]);
            let aircraft = format!("{}\n{}", line1[3], line2[2]);
            let task = format!("{}\n{}", line1[4], line2[3]);

            flights.push(PackageElement {
                is_primary,
                callsign,
                flight,
                role,
                aircraft,
                task,
            })
        }

        flights
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct ThreatAnalysis<'a>(pub &'a str);

impl<'a> ThreatAnalysis<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let Some(threat) = extract_group(briefing, "Threat Analysis") else {
            return Default::default();
        };

        let threat = threat.splitn(3, '\n').last().unwrap_or("");

        Self(threat)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Steerpoint<'a> {
    pub steerpoint: usize,
    pub description: Option<&'a str>,
    pub time: Option<&'a str>,
    pub distance: Option<f64>,
    pub heading: Option<usize>,
    pub cas: Option<usize>,
    pub altitude: Option<&'a str>,
    pub action: Option<&'a str>,
    pub form: Option<&'a str>,
    pub comments: Option<&'a str>,
}

impl<'a> Steerpoint<'a> {
    pub fn from_briefing(briefing: &'a str) -> Vec<Self> {
        let Some(steerpoints) = extract_group(briefing, "Steerpoints") else {
            return Default::default();
        };

        let lines: Vec<&str> = steerpoints
            .lines()
            .skip(4)
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        let mut steerpoints = vec![];
        for line in lines {
            let mut values = line.split('\t').map(str::trim).filter(|l| !l.is_empty());
            let Some(index) = values.next() else { continue };
            let index: usize = index.parse().unwrap();

            let Some(description) = values.next() else {
                continue;
            };
            let description = to_option(description);

            let Some(time) = values.next() else { continue };
            let time = to_option(time);

            let Some(distance) = values.next() else {
                continue;
            };
            let distance: Option<f64> = distance.parse().ok();

            let Some(heading) = values.next() else {
                continue;
            };
            let heading: Option<usize> = heading.parse().ok();

            let Some(cas) = values.next() else { continue };
            let cas: Option<usize> = cas.parse().ok();
            let Some(altitude) = values.next() else {
                continue;
            };
            let altitude = to_option(altitude);
            let Some(action) = values.next() else {
                continue;
            };
            let action = to_option(action);
            let Some(form) = values.next() else { continue };
            let form = to_option(form);
            let Some(comments) = values.next() else {
                continue;
            };
            let comments = to_option(comments);

            steerpoints.push(Self {
                steerpoint: index,
                description,
                time,
                distance,
                heading,
                cas,
                altitude,
                action,
                form,
                comments,
            });
        }

        steerpoints
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Comm<'a> {
    pub agency: &'a str,

    pub callsign: Option<&'a str>,
    pub uhf: Option<&'a str>,
    pub vhf: Option<&'a str>,
    pub notes: Option<&'a str>,
}

impl<'a> Comm<'a> {
    pub fn from_briefing(briefing: &'a str) -> Vec<Self> {
        let Some(commladder) = extract_group(briefing, "Comm Ladder") else {
            return Default::default();
        };

        let lines: Vec<&str> = commladder.lines().skip(4).collect();

        let mut commladder = vec![];
        for line in lines {
            let mut values = line.split('\t').map(str::trim).filter(|l| !l.is_empty());
            let Some(agency) = values.next() else {
                continue;
            };
            let Some(callsign) = values.next() else {
                continue;
            };
            let callsign = to_option(callsign);

            let Some(uhf) = values.next() else {
                continue;
            };
            let uhf = to_option(uhf);

            let Some(vhf) = values.next() else {
                continue;
            };
            let vhf = to_option(vhf);

            let Some(notes) = values.next() else {
                continue;
            };
            let notes = to_option(notes);

            commladder.push(Self {
                agency: agency.trim_matches(|f| f == ':'),
                callsign,
                uhf,
                vhf,
                notes,
            });
        }

        commladder
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct IffInitial<'a> {
    pub modes_active: &'a str,
    pub codes: Vec<&'a str>,
    pub m4_validity_time_until: Vec<&'a str>,
    pub iff_policy: Vec<&'a str>, // M1, M2, M3
    pub code_change_setting: &'a str,
}

impl<'a> IffInitial<'a> {
    fn from_iff(iff: &'a str) -> Self {
        let lines: Vec<&str> = iff.lines().skip(3).take(4).collect();
        let modes_active = lines
            .first()
            .unwrap_or(&"")
            .split('\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .nth(1)
            .unwrap_or("")
            .splitn(2, ':')
            .last()
            .unwrap_or("")
            .trim();

        let codes = lines
            .first()
            .unwrap_or(&"")
            .splitn(5, '\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .nth(3)
            .unwrap_or("")
            .split('\t')
            .map(str::trim)
            .collect::<Vec<&'a str>>();

        let m4_validity_time_until = lines
            .get(1)
            .unwrap_or(&"")
            .split('\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .skip(1)
            .collect::<Vec<&'a str>>();

        let iff_policy = lines
            .get(2)
            .unwrap_or(&"")
            .split('\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .skip(1)
            .collect::<Vec<&'a str>>();

        let code_change_setting = lines
            .get(3)
            .unwrap_or(&"")
            .split('\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .nth(1)
            .unwrap_or("");

        Self {
            modes_active,
            codes,
            m4_validity_time_until,
            iff_policy,
            code_change_setting,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Iff<'a> {
    pub initial: IffInitial<'a>,
    pub time_events: Vec<Vec<&'a str>>,
    pub pos_events: Vec<&'a str>,
}

impl<'a> Iff<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let Some(iff) = extract_group(briefing, "Iff") else {
            return Default::default();
        };
        let initial = IffInitial::from_iff(iff);

        let time_events = iff
            .lines()
            .skip(9)
            .take(5)
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| l.split('\t').collect::<Vec<&'a str>>())
            .collect::<Vec<Vec<&'a str>>>();

        let pos_events = iff
            .lines()
            .last()
            .unwrap_or("")
            .split('\t')
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<&'a str>>();

        Self {
            initial,
            time_events,
            pos_events,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Ordnance<'a> {
    pub flights: Vec<Vec<Vec<&'a str>>>,
}

impl<'a> Ordnance<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let mut flights = vec![];

        let Some(ordnance) = extract_group(briefing, "Ordnance") else {
            return Default::default();
        };

        let lines = ordnance.lines().skip(3).map(str::trim);

        let mut flight: Option<Vec<Vec<&'a str>>> = None;
        for line in lines {
            if line.is_empty() {
                // flush
                if let Some(fl) = flight {
                    flights.push(fl);

                    flight = None;
                }
                continue;
            }

            if flight.is_none() {
                let mut l = line
                    .split('\t')
                    .map(|l| l.trim_matches(|c: char| c == '-' || c.is_whitespace()))
                    .skip(1);

                let mut map = Vec::new();

                for callsign in l.by_ref() {
                    map.push(vec![callsign]);
                }

                flight = Some(map);
                continue;
            }

            let l = line.split('\t').map(str::trim);

            for (i, ord) in l.enumerate() {
                if let Some(map) = flight.as_mut() {
                    map[i].push(ord);
                }
            }
        }

        // flush the last group
        if let Some(fl) = flight {
            flights.push(fl);
        }

        Self { flights }
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Weather<'a>(pub Vec<Vec<&'a str>>);

impl<'a> Weather<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let Some(weather) = extract_group(briefing, "Weather") else {
            return Default::default();
        };

        let weather: Vec<Vec<&str>> = weather
            .lines()
            .skip(2)
            .map(str::trim)
            .filter(|&l| !l.is_empty())
            .map(|l| l.split('\t').map(str::trim).collect::<Vec<_>>())
            .collect();

        Self(weather)
    }
}

// Support
#[derive(Default, Debug, Clone, Serialize)]
pub struct Support<'a>(pub Vec<Vec<&'a str>>);

impl<'a> Support<'a> {
    pub fn from_briefing(briefing: &'a str) -> Self {
        let Some(support) = extract_group(briefing, "Support") else {
            return Default::default();
        };

        let support: Vec<Vec<&str>> = support
            .lines()
            .skip(4)
            .map(str::trim)
            .filter(|&l| !l.is_empty())
            .map(|l| {
                l.split('\t')
                    .map(|l| l.trim_matches(|c: char| c.is_whitespace() || c == ':'))
                    .collect::<Vec<_>>()
            })
            .collect();

        Self(support)
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct RulesOfEngagement(pub String);

impl RulesOfEngagement {
    pub fn from_briefing(briefing: &str) -> Self {
        let Some(roe) = extract_group(briefing, "Rules of Engagement") else {
            return Default::default();
        };

        let lines: Vec<&str> = roe
            .lines()
            .skip(2)
            .map(str::trim)
            .filter(|&l| !l.is_empty())
            .collect();

        let roe = lines.join("\n");

        Self(roe)
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Emergency(pub String);

impl Emergency {
    pub fn from_briefing(briefing: &str) -> Self {
        let Some(emergency) = extract_group(briefing, "Emergency Procedures") else {
            return Default::default();
        };

        let lines: Vec<&str> = emergency.lines().skip(2).map(str::trim).collect();

        let emergency = lines.join("\n");

        Self(emergency)
    }
}

fn to_option(str: &str) -> Option<&str> {
    if str == "--" || str == "None" {
        return None;
    }
    Some(str)
}

fn extract_group<'a>(text: &'a str, group_name: &str) -> Option<&'a str> {
    let mut current_line_start = 0;
    let mut current_group_start = 0;

    for (i, c) in text.chars().enumerate() {
        if current_line_start > i {
            continue;
        }

        if !text[current_line_start..i].starts_with('\t')
            && text[current_line_start..i + 1].ends_with('\n')
            && !text[current_line_start..i + 1].trim().is_empty()
        {
            if text[current_line_start..i].starts_with(group_name) {
                current_group_start = current_line_start;
            } else if current_group_start != 0 {
                return Some(&text[current_group_start..current_line_start - 1]);
            }
        }

        if c == '\n' {
            current_line_start = i + 1;
        }
    }

    None
}
