#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Viet+ — Launchpad PPA Signer & Uploader (Pure Python replacement for dput/debsign)

import os
import sys
import hashlib
import subprocess
import ftplib

GPG_KEY_ID = "4A5CEF12BDC467CE9E613E390D08C5F97EDA7334"
DEFAULT_PPA = "khoavo93/vietc"
LAUNCHPAD_FTP = "ppa.launchpad.net"

def calc_hashes(filepath):
    size = os.path.getsize(filepath)
    with open(filepath, "rb") as f:
        data = f.read()
    md5 = hashlib.md5(data).hexdigest()
    sha1 = hashlib.sha1(data).hexdigest()
    sha256 = hashlib.sha256(data).hexdigest()
    return md5, sha1, sha256, size

def clearsign(filepath, key_id=GPG_KEY_ID):
    signed_path = filepath + ".signed"
    cmd = [
        "gpg", "--batch", "--yes",
        "--default-key", key_id,
        "--clearsign",
        "--output", signed_path,
        filepath
    ]
    subprocess.run(cmd, check=True)
    os.replace(signed_path, filepath)

def sign_and_fix_changes(changes_path, key_id=GPG_KEY_ID):
    base_dir = os.path.dirname(os.path.abspath(changes_path))
    
    # 1. Sign the .dsc file first
    with open(changes_path, "r") as f:
        lines = f.readlines()
    
    dsc_filename = None
    for line in lines:
        parts = line.strip().split()
        if len(parts) >= 2 and parts[-1].endswith(".dsc"):
            dsc_filename = parts[-1]
            break
            
    if dsc_filename:
        dsc_path = os.path.join(base_dir, dsc_filename)
        print(f"Signing DSC: {dsc_filename}...")
        clearsign(dsc_path, key_id)
        
        # Recalculate .dsc hashes after signing
        dsc_md5, dsc_sha1, dsc_sha256, dsc_size = calc_hashes(dsc_path)
        
        new_lines = []
        for line in lines:
            if line.strip().endswith(dsc_filename):
                parts = line.strip().split()
                if len(parts) == 3:  # Sha1 or Sha256 line: <hash> <size> <name>
                    h = dsc_sha1 if len(parts[0]) == 40 else dsc_sha256
                    new_lines.append(f" {h} {dsc_size} {dsc_filename}\n")
                elif len(parts) == 5:  # Files line: <md5> <size> <section> <priority> <name>
                    new_lines.append(f" {dsc_md5} {dsc_size} {parts[2]} {parts[3]} {dsc_filename}\n")
                else:
                    new_lines.append(line)
            else:
                new_lines.append(line)
                
        with open(changes_path, "w") as f:
            f.writelines(new_lines)
            
    # 2. Sign the .changes file
    print(f"Signing Changes: {os.path.basename(changes_path)}...")
    clearsign(changes_path, key_id)

def upload(changes_path, ppa=DEFAULT_PPA, host=LAUNCHPAD_FTP):
    base_dir = os.path.dirname(os.path.abspath(changes_path))
    with open(changes_path, "r") as f:
        content = f.read()
        
    in_files = False
    files_to_upload = []
    for line in content.splitlines():
        if line.startswith("Files:"):
            in_files = True
            continue
        if in_files:
            if line.startswith(" ") or line.startswith("\t"):
                parts = line.strip().split()
                if len(parts) >= 5:
                    files_to_upload.append(parts[4])
            else:
                break
                
    files_to_upload.append(os.path.basename(changes_path))
    target_dir = f"~{ppa}/ubuntu"
    
    print(f"\n=== Uploading package to Launchpad PPA ({ppa}) ===")
    ftp = ftplib.FTP(host)
    ftp.login("anonymous", "anonymous@")
    
    for fname in files_to_upload:
        fpath = os.path.join(base_dir, fname)
        remote_path = f"{target_dir}/{fname}"
        print(f"  -> Uploading {fname} ({os.path.getsize(fpath)} bytes)...")
        with open(fpath, "rb") as fp:
            ftp.storbinary(f"STOR {remote_path}", fp)
            
    ftp.quit()
    print("\n✅ Upload complete! Launchpad will build the binaries shortly.")
    print(f"Check build status at: https://launchpad.net/~{ppa}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path_to_source.changes> [ppa_name]")
        sys.exit(1)
        
    changes_file = sys.argv[1]
    ppa_name = sys.argv[2] if len(sys.argv) > 2 else DEFAULT_PPA
    
    sign_and_fix_changes(changes_file)
    upload(changes_file, ppa=ppa_name)
