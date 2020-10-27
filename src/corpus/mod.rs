use crate::inputs::Input;
use crate::utils::Rand;
use crate::Error;
use hashbrown::HashMap;
use std::fmt::Debug;

pub trait TestcaseMetadata: Debug {}

pub trait Testcase: Debug {
    fn load_input(&mut self) -> Result<&Box<dyn Input>, Error>;
    fn is_on_disk(&self) -> bool;
    fn get_filename(&self) -> &str;
    fn get_metadatas(&mut self) -> &mut HashMap<String, Box<dyn TestcaseMetadata>>;
}

/// Corpus with all current testcases
pub trait Corpus: Debug {
    /// Returns the number of elements
    fn count(&self) -> usize;

    fn add(&mut self, entry: Box<dyn Testcase>);

    /// Removes an entry from the corpus, returning it if it was present.
    fn remove(&mut self, entry: &dyn Testcase) -> Option<Box<dyn Testcase>>;

    /// Gets a random entry
    fn random_entry(&mut self) -> Result<&Box<dyn Testcase>, Error>;

    /// Gets the next entry
    fn get(&mut self) -> Result<&Box<dyn Testcase>, Error>;
}

#[derive(Debug)]
pub struct RandomCorpus<'a> {
    rand: &'a mut dyn Rand,
    entries: Vec<Box<dyn Testcase>>,
    dir_path: String,
}

impl Corpus for RandomCorpus<'_> {
    /// Returns the number of elements
    fn count(&self) -> usize {
        self.entries.len()
    }

    fn add(&mut self, entry: Box<dyn Testcase>) {
        self.entries.push(entry);
    }

    /// Removes an entry from the corpus, returning it if it was present.
    fn remove(&mut self, entry: &dyn Testcase) -> Option<Box<dyn Testcase>> {
        let mut i: usize = 0;
        let mut found = false;
        for x in &self.entries {
            i = i + 1;
            if x.as_ref() as *const _ == entry as *const _ {
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
        Some(self.entries.remove(i))
    }

    /// Gets a random entry
    fn random_entry(&mut self) -> Result<&Box<dyn Testcase>, Error> {
        let id = self.rand.below(self.entries.len() as u64) as usize;
        Ok(self.entries.get_mut(id).unwrap())
    }

    /// Gets the next entry
    fn get(&mut self) -> Result<&Box<dyn Testcase>, Error> {
        self.random_entry()
    }
}

impl RandomCorpus<'_> {
    pub fn new<'a>(rand: &'a mut dyn Rand, dir_path: &str) -> RandomCorpus<'a> {
        RandomCorpus {
            dir_path: dir_path.to_owned(),
            entries: vec![],
            rand: rand,
        }
    }
}

/// A queue-like corpus
#[derive(Debug)]
pub struct QueueCorpus<'a> {
    random_corpus: RandomCorpus<'a>,
    pos: usize,
    cycles: u64,
}

impl Corpus for QueueCorpus<'_> {
    /// Returns the number of elements
    fn count(&self) -> usize {
        self.random_corpus.count()
    }

    fn add(&mut self, entry: Box<dyn Testcase>) {
        self.random_corpus.add(entry);
    }

    /// Removes an entry from the corpus, returning it if it was present.
    fn remove(&mut self, entry: &dyn Testcase) -> Option<Box<dyn Testcase>> {
        self.random_corpus.remove(entry)
    }

    /// Gets a random entry
    fn random_entry(&mut self) -> Result<&Box<dyn Testcase>, Error> {
        self.random_corpus.random_entry()
    }

    /// Gets the next entry
    fn get(&mut self) -> Result<&Box<dyn Testcase>, Error> {
        if self.count() == 0 {
            return Err(Error::Unknown); // TODO(andrea) why unknown? use EmptyContainerError or similar
        }
        self.pos = self.pos + 1;
        if self.pos >= self.count() {
            self.cycles = self.cycles + 1;
            self.pos = 0;
        }
        Ok(self.random_corpus.entries.get_mut(self.pos).unwrap())
    }
}

impl QueueCorpus<'_> {
    pub fn new<'a>(rand: &'a mut dyn Rand, dir_path: &str) -> QueueCorpus<'a> {
        QueueCorpus {
            random_corpus: RandomCorpus::new(rand, dir_path),
            cycles: 0,
            pos: 0,
        }
    }

    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

#[derive(Debug, Default)]
struct SimpleTestcase {
    is_on_disk: bool,
    filename: String,
    metadatas: HashMap<String, Box<dyn TestcaseMetadata>>,
}

impl Testcase for SimpleTestcase {
    fn load_input(&mut self) -> Result<&Box<dyn Input>, Error> {
        // TODO: Implement
        Err(Error::Unknown)
    }

    fn is_on_disk(&self) -> bool {
        self.is_on_disk
    }

    fn get_filename(&self) -> &str {
        &self.filename
    }

    fn get_metadatas(&mut self) -> &mut HashMap<String, Box<dyn TestcaseMetadata>> {
        &mut self.metadatas
    }
}

impl SimpleTestcase {
    fn new(filename: &str) -> Self {
        SimpleTestcase {
            filename: filename.to_owned(),
            is_on_disk: false,
            metadatas: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::corpus::Corpus;
    use crate::corpus::QueueCorpus;
    use crate::corpus::SimpleTestcase;
    use crate::utils::Xoshiro256StarRand;

    #[test]
    fn test_queuecorpus() {
        let mut rand = Xoshiro256StarRand::new();
        let mut q = QueueCorpus::new(&mut rand, "fancy/path");
        q.add(Box::new(SimpleTestcase::new("fancyfile")));
        let filename = q.get().unwrap().get_filename().to_owned();
        assert_eq!(filename, q.get().unwrap().get_filename());
        assert_eq!(filename, "fancyfile");
    }
}
