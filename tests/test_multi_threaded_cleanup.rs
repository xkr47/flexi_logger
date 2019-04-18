#[cfg(feature = "ziplogs")]
mod d {
    use chrono::Local;
    use flexi_logger::{Cleanup, Criterion, Duplicate, LogSpecification, Logger, Naming, Record};
    use glob::glob;
    use log::*;
    use std::io;
    use std::ops::Add;
    use std::thread::{self, JoinHandle};
    use std::time;

    const NO_OF_THREADS: usize = 5;
    const NO_OF_LOGLINES_PER_THREAD: usize = 100_000;
    const ROTATE_OVER_SIZE: u64 = 3_000_000;
    const NO_OF_LOG_FILES: usize = 2;
    const NO_OF_ZIP_FILES: usize = 5;

    #[test]
    fn multi_threaded() {
        // we use a special log line format that starts with a special string so that it is easier to
        // verify that all log lines are written correctly

        let start = Local::now();
        let directory = define_directory();
        let mut reconf_handle = Logger::with_str("debug")
            .log_to_file()
            .format(test_format)
            .duplicate_to_stderr(Duplicate::Info)
            .directory(directory.clone())
            .rotate(
                Criterion::Size(ROTATE_OVER_SIZE),
                Naming::Timestamps,
                Cleanup::KeepLogAndZipFiles(NO_OF_LOG_FILES, NO_OF_ZIP_FILES),
            )
            .start()
            .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
        info!(
        "create a huge number of log lines with a considerable number of threads, verify the log"
    );

        let worker_handles = start_worker_threads(NO_OF_THREADS);
        let new_spec = LogSpecification::parse("trace").unwrap();
        thread::Builder::new()
            .spawn(move || {
                thread::sleep(time::Duration::from_millis(1000));
                reconf_handle.set_new_spec(new_spec);
                0 as u8
            })
            .unwrap();

        wait_for_workers_to_close(worker_handles);

        let delta = Local::now().signed_duration_since(start).num_milliseconds();
        debug!(
            "Task executed with {} threads in {}ms.",
            NO_OF_THREADS, delta
        );
        verify_logs(&directory);
    }

    // Starts given number of worker threads and lets each execute `do_work`
    fn start_worker_threads(no_of_workers: usize) -> Vec<JoinHandle<u8>> {
        let mut worker_handles: Vec<JoinHandle<u8>> = Vec::with_capacity(no_of_workers);
        trace!("Starting {} worker threads", no_of_workers);
        for thread_number in 0..no_of_workers {
            trace!("Starting thread {}", thread_number);
            worker_handles.push(
                thread::Builder::new()
                    .name(thread_number.to_string())
                    .spawn(move || {
                        do_work(thread_number);
                        0 as u8
                    })
                    .unwrap(),
            );
        }
        trace!("All {} worker threads started.", worker_handles.len());
        worker_handles
    }

    fn do_work(thread_number: usize) {
        trace!("({})     Thread started working", thread_number);
        trace!("ERROR_IF_PRINTED");
        for idx in 0..NO_OF_LOGLINES_PER_THREAD {
            debug!("({})  writing out line number {}", thread_number, idx);
        }
        trace!("MUST_BE_PRINTED");
    }

    fn wait_for_workers_to_close(worker_handles: Vec<JoinHandle<u8>>) {
        for worker_handle in worker_handles {
            worker_handle
                .join()
                .unwrap_or_else(|e| panic!("Joining worker thread failed: {:?}", e));
        }
        trace!("All worker threads joined.");
    }

    fn define_directory() -> String {
        format!(
            "./log_files/mt_logs/{}",
            Local::now().format("%Y-%m-%d_%H-%M-%S")
        )
    }

    pub fn test_format(w: &mut io::Write, record: &Record) -> io::Result<()> {
        write!(
            w,
            "XXXXX [{}] T[{:?}] {} [{}:{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S%.6f %:z"),
            thread::current().name().unwrap_or("<unnamed>"),
            record.level(),
            record.file().unwrap_or("<unnamed>"),
            record.line().unwrap_or(0),
            &record.args()
        )
    }

    fn verify_logs(directory: &str) {
        // Since the cleanup deleted log files, we just can confirm that the correct number of
        // log files and zip files exist

        let basename = String::from(directory).add("/").add(
            &std::path::Path::new(&std::env::args().next().unwrap())
            .file_stem().unwrap(/*cannot fail*/)
            .to_string_lossy().to_string(),
        );

        let fn_pattern = String::with_capacity(180)
            .add(&basename)
            .add("_r[0-9][0-9]*.");

        let log_pattern = fn_pattern.clone().add("log");
        println!("log_pattern = {}", log_pattern);
        let no_of_log_files = glob(&log_pattern)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<std::path::PathBuf>>()
            .len();

        let zip_pattern = fn_pattern.add("zip");
        let no_of_zip_files = glob(&zip_pattern)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<std::path::PathBuf>>()
            .len();

        assert_eq!(no_of_log_files, NO_OF_LOG_FILES);
        assert_eq!(no_of_zip_files, NO_OF_ZIP_FILES);

        info!("Found correct number of log and zip files");
    }
}
