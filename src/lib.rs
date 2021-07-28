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
    /// Per Maven documentation, group ID uniquely identifies the project among all the other
    /// projects. It should, but not required to, follow [Java package name rules][java-naming].
    ///
    /// Essentially, it's just an identifier of the group that created the project (their owned
    /// domain) and at times, name of the projects group (like `utilities`, `core`).
    ///
    /// [java-naming]: https://docs.oracle.com/javase/specs/jls/se6/html/packages.html#7.7
    ///
    /// # Examples
    ///
    /// - `org.apache.commons`
    /// - `com.mojang`
    /// - `io.github.brawaru.plugins`
    ///
    /// # Usage
    ///
    /// Group ID is used to resolve location of the artifact directory, both locally and remotely.
    /// All the dots are replaced with case-specific path separator. For example, `com.mojang` will
    /// became `com/mojang/` when resolving remote path to the artifact.
    pub group_id: String,

    /// Per Maven documentation, artifact ID is generally the name that the project is known by.
    /// It has to be all lower-case and contain no "strange symbols" (dashes are allowed).
    ///
    /// # Examples
    ///
    /// - `commons-lang3`
    /// - `jackson-core`
    /// - `shadow`
    ///
    /// # Usage
    ///
    /// Paired with Group ID, artifact ID should generate unique identifier for the whole project.
    /// Artifact ID is also used to determine name of the file (excluding version and classifier)
    /// and the subdirectory that contains all the versions directories.
    pub artifact_id: String,

    /// Per Maven documentation, this is the version of the artifact itself. It's recommended to
    /// follow [semantic versioning](https://semver.org/), but again, not required to (not outside
    /// the Maven Central Repository, at least).
    ///
    /// # Library note
    ///
    /// This library follows semantic versioning, therefore it will denote version label to the
    /// version itself. To denote version label, at last dash (`-`) within the version, the split
    /// will be made, where everything before the dash will make it into the [`version`][0], and
    /// anything that follows it into the [`version_label`][1]. To get complete version, including
    /// the label, you'll have to use [`full_version`][2] method.
    ///
    /// [0]: Coordinates::version
    /// [1]: Coordinates::version_label
    /// [2]: Coordinates::full_version
    ///
    /// # Usage
    ///
    /// Version used both in the name of the artifact file, as well as separating directory.
    pub version: String,

    /// Denoted by last dash in version part of the coordinates, label for this version (if any).
    ///
    /// To get complete version, including the label itself, use [`full_version`][0] method.
    ///
    /// [0]: Coordinates::full_version
    ///
    /// # Examples
    ///
    /// - `SNAPSHOT` to refer to constantly changing snapshot.
    /// - `rc1` to refer to first release candidate.
    ///
    /// # Usage
    ///
    /// Version label used in the name of the artifact file, following the version.
    pub version_label: Option<String>,

    /// Packaging is essentially an extension of the artifact. If not specified in coordinates,
    /// it is assumed to be `jar`.
    ///
    /// # Examples
    ///
    /// - `jar` is the default, when not specified.
    /// - `jar.sha1` to get file with SHA-1 for the original JAR file.
    /// - `pom` to get the POM of the artifact.
    ///
    /// # Usage
    ///
    /// Packaging is used as an extension when resolving the artifact file name.
    pub packaging: String,

    /// Per Maven documentation, a classifier distinguishes artifacts that were built from the
    /// same POM but differ in content.
    ///
    /// # Examples
    ///
    /// - `jdk8`
    /// - `jdk11`
    ///
    /// # Usage
    ///
    /// Classifier is added after the version number when resolving the artifact file name.
    pub classifier: Option<String>,
}

impl ToString for Coordinates {
    fn to_string(&self) -> String {
        // $groupId:$artifactId:$version:$packaging:$classifier

        let mut string = String::new();

        string += &self.group_id;

        string += MAVEN_COORDINATES_SPLITTER;
        string += &self.artifact_id;

        string += MAVEN_COORDINATES_SPLITTER;
        string += self.full_version().as_str();

        if !self.packaging.eq(MAVEN_STANDARD_PACKAGING) || self.classifier.is_some() {
            string += MAVEN_COORDINATES_SPLITTER;
            string += &self.packaging;

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
        file_name += &self.packaging;

        file_name
    }

    /// Converts coordinates to the path string with default separator (`/`).
    ///
    /// returns: String
    pub fn to_path(&self) -> String {
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

        path.push_str(&self.artifact_id);
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

        maven_location += &self.to_path();

        maven_location
    }
}
