use std::io::Write;
use tempfile::TempPath;

#[inline]
fn run_cmd(args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new(env!("CARGO_BIN_EXE_sig")).args(args).output()
}

fn write_temp_file(lines: &[impl AsRef<str>]) -> std::io::Result<TempPath> {
    let mut file = tempfile::NamedTempFile::new()?;
    for line in lines {
        writeln!(file, "{}", line.as_ref())?;
    }
    file.flush()?;
    Ok(file.into_temp_path())
}

#[test]
fn test_count() -> std::io::Result<()> {
    let output = run_cmd(&["count", "0x00"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains('1'));

    let output = run_cmd(&["count", "0xABCDEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains('3'));

    let output = run_cmd(&["count", "ABCDEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains('3'));

    let output = run_cmd(&["count", "A??B??C"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains('4'));

    let output = run_cmd(&["count", ""])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains('0'));

    let output = run_cmd(&["count", "GX"])?;
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[test]
fn test_format() -> std::io::Result<()> {
    let output = run_cmd(&["format", "0x00"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("00"));

    let output = run_cmd(&["format", "0xABCDEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD EF"));

    let output = run_cmd(&["format", "ABCDEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD EF"));

    let output = run_cmd(&["format", "0x???BC"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("?? ?B 0C"));

    let output = run_cmd(&["format", "A??B??CD"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("A? ?B ?? CD"));

    let output = run_cmd(&["format", "GX"])?;
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[test]
fn test_merge() -> std::io::Result<()> {
    let output = run_cmd(&["merge", "0xABCD", "0xABCD"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD"));

    let output = run_cmd(&["merge", "0xABCD", "0xABCC"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB C?"));

    let output = run_cmd(&["merge", "0xABCD", "0x?BCD"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("?B CD"));

    let output = run_cmd(&["merge", "0xABCD", "0x??CD"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("?? CD"));

    let output = run_cmd(&["merge", "0xABCD", "0xABCDEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD ??"));

    let output = run_cmd(&["merge", "A B C D E F", "AB CD"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD ??"));

    let output = run_cmd(&["merge"])?;
    assert_eq!(output.status.code(), Some(0));

    let output = run_cmd(&["merge", "AB CD", "GX"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    let err = String::from_utf8_lossy(output.stderr.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD"));
    assert!(err.contains("Skipped invalid pattern"));

    let output = run_cmd(&["merge", "0xABCDEF12", "0xABCCEF"])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB C? EF ??"));
    Ok(())
}

#[test]
fn test_merge_file() -> std::io::Result<()> {
    let file = write_temp_file(&["ABCD", "ABCD"])?;
    let output = run_cmd(&["merge", "--file", file.to_str().unwrap()])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB CD"));

    let file = write_temp_file(&["0xABCD", "0xABCC"])?;
    let output = run_cmd(&["merge", "--file", file.to_str().unwrap()])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB C?"));

    let file = write_temp_file(&["0xABCC"])?;
    let output = run_cmd(&["merge", "ABCD", "--file", file.to_str().unwrap()])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB C?"));

    let file = write_temp_file(&["0xABCDFF"])?;
    let output = run_cmd(&["merge", "ABCD", "ABCC", "--file", file.to_str().unwrap()])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("AB C? ??"));

    let file = write_temp_file(&["0xFFFFFF"])?;
    let output = run_cmd(&["merge", "ABCD", "ABCC", "--file", file.to_str().unwrap()])?;
    let str = String::from_utf8_lossy(output.stdout.as_slice());
    assert_eq!(output.status.code(), Some(0));
    assert!(str.contains("?? ?? ??"));
    Ok(())
}
