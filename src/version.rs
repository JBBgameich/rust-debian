use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct VersionElement {
    pub alpha: String,
    pub numeric: u64,
}

impl Ord for VersionElement {
    fn cmp(&self, other: &VersionElement) -> Ordering {
        assert!(self.alpha.is_empty());
        assert!(other.alpha.is_empty());
        // FIXME: compare alpha, first!
        self.numeric.cmp(&other.numeric)
    }
}

impl fmt::Display for VersionElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.alpha, self.numeric)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct VersionPart {
    pub elements: Vec<VersionElement>,
}

impl VersionPart {
    fn count_elements(&self) -> usize {
        self.elements.len()
    }
}

impl fmt::Display for VersionPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .elements
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .concat();
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Version {
    pub epoch: u32,
    pub upstream_version: VersionPart,
    pub debian_revision: VersionPart,
}

#[derive(Debug)]
pub struct ParseError {
    pub pos: i32,
    pub msg: String,
}

impl Version {
    pub fn parse_part(s: &str) -> Result<VersionPart, ParseError> {
        if s.is_empty() {
            return Ok(VersionPart { elements: vec![] });
        }
        let mut elements: Vec<VersionElement> = vec![];
        let mut in_numeric_part = false;
        let mut cur = VersionElement {
            alpha: "".to_string(),
            numeric: 0,
        };
        for c in s.chars() {
            match (in_numeric_part, c.is_digit(10)) {
                (false, false) => cur.alpha.push(c),
                (_, true) => {
                    in_numeric_part = true;
                    cur.numeric *= 10;
                    cur.numeric += (c as u64) - ('0' as u64);
                }
                (true, false) => {
                    elements.push(cur);
                    in_numeric_part = false;
                    cur = VersionElement {
                        alpha: "".to_string(),
                        numeric: 0,
                    };
                    cur.alpha.push(c);
                }
            }
        }
        elements.push(cur);
        Ok(VersionPart { elements })
    }

    pub fn parse(s: &str) -> Result<Version, ParseError> {
        let first_colon = s.find(':');
        let last_dash = s.rfind('-');

        let epoch = match first_colon {
            Some(l) => {
                let epoch_str = &s[..l];
                match u32::from_str(epoch_str) {
                    Ok(v) => Some((l, v)),
                    Err(_) => {
                        return Err(ParseError {
                            pos: 0,
                            msg: "Expected a numeric epoch.".to_string(),
                        })
                    }
                }
            }
            None => None,
        };

        Ok(match (epoch, last_dash) {
            (Some((l, epoch)), Some(r)) => Version {
                epoch,
                upstream_version: try!(Version::parse_part(&s[l + 1..r])),
                debian_revision: try!(Version::parse_part(&s[r + 1..])),
            },
            (Some((l, epoch)), None) => Version {
                epoch,
                upstream_version: try!(Version::parse_part(&s[l + 1..])),
                debian_revision: VersionPart { elements: vec![] },
            },
            (None, Some(r)) => Version {
                epoch: 0,
                upstream_version: try!(Version::parse_part(&s[..r])),
                debian_revision: try!(Version::parse_part(&s[r + 1..])),
            },
            (None, None) => Version {
                epoch: 0,
                upstream_version: try!(Version::parse_part(&s[..])),
                debian_revision: VersionPart { elements: vec![] },
            },
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        let epoch_cmp = self.epoch.cmp(&other.epoch);
        match epoch_cmp {
            Ordering::Equal => Ordering::Equal,
            ord => ord,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.epoch, self.debian_revision.count_elements()) {
            (0, 0) => write!(f, "{}", &self.upstream_version),
            (0, _) => write!(
                f,
                "{}-{}",
                &self.upstream_version, &self.debian_revision
            ),
            (_, 0) => write!(f, "{}:{}", self.epoch, &self.upstream_version),
            (_, _) => write!(
                f,
                "{}:{}-{}",
                self.epoch, &self.upstream_version, &self.debian_revision
            ),
        }
    }
}
