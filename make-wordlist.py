import bs4
import requests
from bs4 import BeautifulSoup


def words_from_page(url: str) -> set[str]:
    request = requests.get(url)
    soup = BeautifulSoup(request.text, 'html.parser')
    return {word.get_text().strip() for word in soup.find_all(class_='wordWrapper')}


def main() -> None:
    baseurl = 'https://scrabblewordfinder.org'
    homepage = f'{baseurl}/word-list'
    request = requests.get(homepage)
    soup = BeautifulSoup(request.text, 'html.parser')
    words = set()
    result: bs4.Tag
    for result in soup.find_all(class_='result'):
        header = result.find('h3')
        assert header is not None
        if header.get_text().strip().lower() == 'others':
            break
        for link in result.find_all('a'):
            rel_ref: str = link.attrs['href']
            ref = rel_ref if rel_ref.startswith(baseurl) else f'{baseurl}/{rel_ref}'
            print(f'Gathering from {ref}')
            words |= words_from_page(ref)
    with open('wordlist.txt', 'w+') as f:
        f.write('\n'.join(sorted(words)))



if __name__ == "__main__":
    main()
