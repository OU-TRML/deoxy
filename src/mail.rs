//! Contains utilities for sending email notifications.

use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
};

/// Send an email to the specified recipients.
// Thanks to BurntSushi.
pub fn mail(
    to: &[impl ToString],
    subject: impl ToString,
    message: impl ToString,
) -> std::io::Result<()> {
    let mut child = Command::new("sendmail")
        .arg("-t")
        .stdin(Stdio::piped())
        .spawn()?;
    {
        let mut buf = BufWriter::new(child.stdin.as_mut().unwrap());
        writeln!(
            &mut buf,
            "\
Subject: {subject}
From: deoxy@hmltn.me
",
            subject = subject.to_string()
        )?;
        for recipient in to {
            writeln!(&mut buf, "To: {}", recipient.to_string())?;
        }
        writeln!(&mut buf)?;
        writeln!(&mut buf, "{}", message.to_string())?;
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(match status.code() {
            None => {
                std::io::Error::new(std::io::ErrorKind::Interrupted, "Email sending interrupted")
            }
            Some(_) => std::io::Error::new(std::io::ErrorKind::Other, status.to_string()),
        })
    }
}
