use std::io::ErrorKind;

/// Maven coordinates part separator.
const MAVEN_COORDINATES_SPLITTER: &str = ":";

/// Standard packaging used by Maven if no packaging is provided in the coordinates.
const MAVEN_STANDARD_PACKAGING: &str = "jar";

/// Splitter used to separate artifact name from version and classifier in file name.
const FILENAME_SPLITTER: &str = "-";

// Splitter used to separate packaging (extension) from the base name of the artifact.
const EXTENSION_SPLITTER: &str = ".";

// Default separator
const DEFAULT_SEPARATOR: char = '/';

/// Standard Maven Coordinates.
#[derive(Debug, Clone)]
pub struct Coordinates {
    group_id: String,
    artifact_id: String,
    version: String,
    version_label: Option<String>,
    packaging: String,
    classifier: Option<String>,
}

impl ToString for Coordinates {
    fn to_string(&self) -> String {
        // $groupId:$artifactId:$version:$packaging:$classifier

        let mut string = String::new();

        string += self.group_id();

        string += MAVEN_COORDINATES_SPLITTER;
        string += self.artifact_id();

        string += MAVEN_COORDINATES_SPLITTER;
        string += self.full_version().as_str();

        if !self.packaging.eq(MAVEN_STANDARD_PACKAGING) || self.classifier.is_some() {
            string += MAVEN_COORDINATES_SPLITTER;
            string += self.packaging();

            if let Some(classifier) = &self.classifier {
                string += MAVEN_COORDINATES_SPLITTER;
                string += classifier;
            }
        }

        string
    }
}

impl Coordinates {

    /// Creates new coordinates struct from the coordinates string.
    ///
    /// # Arguments
    ///
    /// * `coordinates`: Maven coordinates string, which follows the format:
    ///   `$groupId:$artifactId:$version[:$packaging[:$classifier]]`.
    ///
    /// # Returns
    ///
    /// Result<Coordinates, ErrorKind>
    ///
    /// If coordinates string is correct and parsed, this will be `Ok(Coordinates)`, otherwise
    /// `Err(ErrorKind::InvalidInput)` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT").unwrap();
    /// ```
    pub fn new<S: Into<String>>(coordinates: S) -> Result<Self, ErrorKind> {
        let coordinates = coordinates.into();

        let mut parts = coordinates.split(MAVEN_COORDINATES_SPLITTER);

        // $groupId:$packageId:$version-$qualifier:$packaging:$classifier

        let group_id = parts.next();
        let artifact_id = parts.next();
        let version_part = parts.next();

        // Group ID, artifact ID and version are a mandatory
        if group_id.is_none() || artifact_id.is_none() || version_part.is_none() {
            return Err(ErrorKind::InvalidInput);
        }

        let (version, version_qualifier) = Coordinates::split_version(version_part.unwrap());
        let packaging = parts.next();
        let classifier = parts.next();

        Ok(Self {
            group_id: group_id.unwrap().to_string(),
            artifact_id: artifact_id.unwrap().to_string(),
            version: version.to_string(),
            version_label: version_qualifier.map_or(None, |q| Some(q.to_string())),
            packaging: packaging.unwrap_or(MAVEN_STANDARD_PACKAGING).to_string(),
            classifier: classifier.map_or(None, |s| Some(s.to_string())),
        })
    }

    /// Splits version into the slices of version itself and the qualifier part.
    ///
    /// # Arguments
    ///
    /// * `version`: Source version string to split
    ///
    /// returns: (&str, Option<&str>)
    fn split_version(version: &str) -> (&str, Option<&str>) {
        if let Some(split_index) = version.rfind(FILENAME_SPLITTER) {
            (&version[..split_index], Some(&version[split_index + 1..]))
        } else {
            (&version, None)
        }
    }

    /// Returns complete version (including the label).
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT").unwrap().full_version();
    /// // => "1.0.0-SNAPSHOT"
    /// ```
    pub fn full_version(&self) -> String {
        let mut full_version = self.version.to_string();

        if let Some(version_qualifier) = &self.version_label {
            full_version += FILENAME_SPLITTER;
            full_version += version_qualifier;
        }

        full_version
    }

    /// Returns base file name for this artifact.
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT:jar:sources").unwrap().file_basename();
    /// // => "artifact-1.0.0-SNAPSHOT-sources"
    /// ```
    pub fn file_basename(&self) -> String {
        let mut file_name = self.artifact_id.to_string();

        file_name += FILENAME_SPLITTER;
        file_name += &self.full_version();

        if let Some(classifier) = &self.classifier {
            file_name += FILENAME_SPLITTER;
            file_name += classifier;
        }

        file_name
    }

    /// Returns complete file name for this artifact.
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT:jar:sources").unwrap().file_name();
    /// // => "artifact-1.0.0-SNAPSHOT-sources.jar"
    /// ```
    pub fn file_name(&self) -> String {
        let mut file_name = self.file_basename().to_string();

        file_name += EXTENSION_SPLITTER;
        file_name += self.packaging();

        file_name
    }

    /// Converts coordinates to the path string with default separator (`/`).
    ///
    /// returns: String
    pub fn as_path(&self) -> String {
        self.as_path_with_separator(DEFAULT_SEPARATOR)
    }

    /// Converts coordinates to the path string with custom separator.
    ///
    /// # Arguments
    ///
    /// * `separator`: path separator.
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    /// let artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT").unwrap();
    /// artifact.as_path_with_separator('\\');
    /// // => "io\\github\\brawaru\\artifact\\1.0.0-SNAPSHOT\\artifact-1.0.0-SNAPSHOT.jar"
    /// ```
    pub fn as_path_with_separator(&self, separator: char) -> String {
        let mut path = String::new();

        for directory in self.group_id.split(".") {
            path.push_str(directory);
            path.push(separator);
        }

        path.push_str(self.artifact_id());
        path.push(separator);

        path.push_str(self.full_version().as_str());
        path.push(separator);

        path.push_str(self.file_name().as_str());

        path
    }

    /// Resolves URL for the artifact using given base Maven server address.
    ///
    /// # Arguments
    ///
    /// * `maven_server`: Address of remote Maven server
    ///
    /// returns: String
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let coords = Coordinates::new("io.github.brawaru:artifact:1.0.0-SNAPSHOT").unwrap();
    /// coords.resolve("https://brawaru.github.io/maven/");
    /// // => "https://brawaru.github.io/maven/io/github/brawaru/artifact/1.0.0-SNAPSHOT/artifact-1.0.0-SNAPSHOT.jar"
    /// ```
    pub fn resolve(&self, maven_location: &str) -> String {
        let mut maven_location = maven_location.to_string();

        if maven_location.chars().last().unwrap_or(' ') != '/' {
            maven_location += "/";
        }

        maven_location += &self.as_path();

        maven_location
    }

    /// Returns group ID part of the coordinates.
    ///
    /// returns: &str
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    /// Returns artifact ID part of the coordinates.
    ///
    /// returns: &str
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Returns version from of the artifact (excluding label).
    ///
    /// returns: &str
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns version label of the artifact.
    ///
    /// returns: &str
    pub fn version_label(&self) -> &Option<String> {
        &self.version_label
    }

    /// Returns packaging of the artifact (e.g. `jar`).
    ///
    /// returns: &str
    pub fn packaging(&self) -> &str {
        &self.packaging
    }

    /// Returns classifier of the artifact.
    ///
    /// returns: &str
    pub fn classifier(&self) -> &Option<String> {
        &self.classifier
    }

    /// Sets the group ID part of the coordinates.
    ///
    /// # Arguments
    ///
    /// * `group_id`: new group ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_group_id("io.github.brawarufixes");
    /// artifact.as_path();
    /// // => "io/github/brawarufixes/artifact/1.0.0/artifact-1.0.0.jar"
    /// ```
    pub fn set_group_id<S: Into<String>>(&mut self, group_id: S) {
        self.group_id = group_id.into();
    }

    /// Sets the artifact ID part of the coordinates.
    ///
    /// # Arguments
    ///
    /// * `artifact_id`: new artifact ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_artifact_id("fixed-artifact");
    /// artifact.as_path();
    /// // => "io/github/brawaru/fixed-artifact/1.0.0/fixed-artifact-1.0.0.jar"
    /// ```
    pub fn set_artifact_id<S: Into<String>>(&mut self, artifact_id: S) {
        self.artifact_id = artifact_id.into();
    }

    /// Sets the version of the artifact by coordinates (excluding label).
    ///
    /// # Arguments
    ///
    /// * `version`: new version of the artifact (excluding label).
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_version("1.0.1");
    /// artifact.as_path();
    /// // => "io/github/brawaru/artifact/1.0.1/artifact-1.0.1.jar"
    /// ```
    pub fn set_version<S: Into<String>>(&mut self, version: S) {
        self.version = version.into();
    }

    /// Sets the version label of the artifact by coordinates (excluding version itself).
    ///
    /// # Arguments
    ///
    /// * `version_qualifier`: new version label (excluding version itself).
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_version_label(Some("-HOTFIX"));
    /// artifact.as_path();
    /// // => "io/github/brawaru/artifact/1.0.0-HOTFIX/artifact-1.0.0-HOTFIX.jar"
    /// ```
    pub fn set_version_label<S: Into<String>>(&mut self, version_qualifier: Option<S>) {
        self.version_label = version_qualifier.map_or(None, |q| Some(q.into()));
    }

    /// Sets the packaging of the artifact by coordinates.
    ///
    /// # Arguments
    ///
    /// * `packaging`: new packaging of the artifact.
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_packaging("jar.sha1");
    /// artifact.as_path();
    /// // => "io/github/brawaru/artifact/1.0.0/artifact-1.0.0.jar.sha1"
    /// ```
    pub fn set_packaging<S: Into<String>>(&mut self, packaging: S) {
        self.packaging = packaging.into();
    }

    /// Sets the classifier of the artifact by coordinates.
    ///
    /// # Arguments
    ///
    /// * `classifier`: new classifier of the artifact.
    ///
    /// # Examples
    ///
    /// ```
    /// use maven_coordinates::Coordinates;
    ///
    /// let mut artifact = Coordinates::new("io.github.brawaru:artifact:1.0.0").unwrap();
    ///
    /// artifact.set_classifier(Some("jdk11"));
    /// artifact.as_path();
    /// // => "io/github/brawaru/artifact/1.0.0/artifact-1.0.0-jdk11.jar"
    /// ```
    pub fn set_classifier<S: Into<String>>(&mut self, classifier: Option<S>) {
        self.classifier = classifier.map_or(None, |s| Some(s.into()));
    }
}
