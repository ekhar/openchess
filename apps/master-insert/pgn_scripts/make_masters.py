import requests
from bs4 import BeautifulSoup
import os
import urllib.parse
from typing import Optional


def is_zip_link(href: Optional[str]) -> bool:
    return href is not None and href.endswith(".zip")


def download_zip_files(
    html_file_path: str, base_url: str = "https://www.pgnmentor.com/"
) -> None:
    # Read the HTML file
    with open(html_file_path, "r") as file:
        html_content = file.read()
    # Parse the HTML content
    soup = BeautifulSoup(html_content, "html.parser")
    # Find all 'a' tags with href attribute ending in '.zip'
    zip_links = soup.find_all("a", href=is_zip_link)
    # Create a directory to store the downloaded files
    if not os.path.exists("downloaded_zips"):
        os.makedirs("downloaded_zips")
    # Download each ZIP file
    for link in zip_links:
        # Get the href attribute
        href = link.get("href")
        if href is None:
            continue
        # Prepend the base URL
        full_url = urllib.parse.urljoin(base_url, href)
        # Get the filename from the URL
        filename = os.path.join("downloaded_zips", os.path.basename(full_url))
        print(f"Downloading: {full_url}")
        # Download the file
        response = requests.get(full_url)
        # Save the file
        with open(filename, "wb") as file:
            file.write(response.content)
        print(f"Saved as: {filename}")
    print("Download complete!")


def download_twic():
    base_url = "https://theweekinchess.com/zips/twic"

    if not os.path.exists("twic_zips"):
        os.makedirs("twic_zips")

    for i in range(920, 1560):
        url = f"{base_url}{i}g.zip"
        filename = os.path.join("twic_zips", f"twic{i}g.zip")

        print(f"Downloading: {url}")
        response = requests.get(url)

        if response.status_code == 200:
            with open(filename, "wb") as file:
                file.write(response.content)
            print(f"Saved as: {filename}")
        else:
            print(f"Failed to download: {url}")

    print("TWIC download complete!")


# Usage
html_file_path = "./site.html"
# download_zip_files(html_file_path)

# Run the new function to download TWIC files
download_twic()
