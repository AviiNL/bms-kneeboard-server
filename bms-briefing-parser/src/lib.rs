use serde::Serialize;

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
        let lines: Vec<String> = briefing
            .lines()
            .skip_while(|l| !l.contains("Mission Overview:"))
            .skip(2)
            .take_while(|l| l.starts_with('\t'))
            .map(|l| l.trim().replace("\t\t", "\t"))
            .filter(|l| !l.is_empty())
            .collect();

        let own = lines[0].split(' ').take(1).collect::<Vec<&str>>()[0].to_owned();

        let lines: Vec<String> = briefing
            .lines()
            .skip_while(|l| !l.contains("Package Elements:"))
            .skip(4)
            .take_while(|l| l.starts_with('\t'))
            .map(|l| l.replace("\t\t", "\t"))
            .filter(|l| !l.is_empty())
            .collect();

        let mut flights = vec![];
        for f in (0..lines.len() - 1).step_by(2) {
            let lines: Vec<&String> = vec![&lines[f], &lines[f + 1]];

            let line1: Vec<&str> = lines[0].split('\t').collect();
            let line2: Vec<&str> = lines[1].split('\t').collect();

            let is_primary = line1[1].contains(&own);

            let callsign = line1[1].to_owned();
            let flight = line1[2].to_owned() + "\n" + line2[2];
            let role = line1[3].to_owned() + "\n" + line2[3];
            let aircraft = line1[4].to_owned() + "\n" + line2[4];
            let task = line1[5].to_owned() + "\n" + line2[5];

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

#[derive(Debug, Clone, Serialize)]
pub struct Steerpoint {
    pub steerpoint: usize,
    pub description: Option<String>,
    pub time: Option<String>,
    pub distance: Option<f64>,
    pub heading: Option<usize>,
    pub cas: Option<usize>,
    pub altitude: Option<String>,
    pub action: Option<String>,
    pub form: Option<String>,
    pub comments: Option<String>,
}

impl Steerpoint {
    pub fn from_briefing(briefing: &str) -> Vec<Self> {
        let lines: Vec<String> = briefing
            .lines()
            .skip_while(|l| !l.contains("Steerpoints:"))
            .skip(4)
            .take_while(|l| l.starts_with('\t'))
            .map(|l| l.trim().replace("\t\t", "\t"))
            .collect();

        let mut steerpoints = vec![];
        for line in lines {
            let mut values = line.split('\t');
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
pub struct Comm {
    pub agency: String,
    pub callsign: Option<String>,
    pub uhf: Option<String>,
    pub vhf: Option<String>,
    pub notes: Option<String>,
}

impl Comm {
    pub fn from_briefing(briefing: &str) -> Vec<Self> {
        let lines: Vec<String> = briefing
            .lines()
            .skip_while(|l| !l.contains("Comm Ladder:"))
            .skip(4)
            .take_while(|l| l.starts_with('\t') || l.trim().is_empty())
            .map(|l| l.trim().replace("\t\t", "\t"))
            .collect();

        let mut commladder = vec![];
        for line in lines {
            let mut values = line.split('\t');
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
                agency: agency.replace(':', ""),
                callsign,
                uhf,
                vhf,
                notes,
            });
        }

        commladder
    }
}

fn to_option(str: &str) -> Option<String> {
    if str == "--" || str == "None" {
        return None;
    }
    Some(str.to_owned())
}
