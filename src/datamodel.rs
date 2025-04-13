pub struct Author {
    pub name: &'static str,
    pub born: &'static str,
}

pub struct Book {
    pub title: &'static str,
    pub author: &'static str,
}

pub static ALL_AUTHORS: &[Author] = &[
    Author {
        name: "Hermann Hesse",
        born: "1877-07-02",
    },
    Author {
        name: "Thomas Mann",
        born: "1875-06-06",
    },
];

pub static ALL_BOOKS: &[Book] = &[
    Book {
        title: "Siddhartha",
        author: "Hermann Hesse",
    },
    Book {
        title: "Das Glasperlenspiel",
        author: "Hermann Hesse",
    },
    Book {
        title: "Zauberberg",
        author: "Thomas Mann",
    },
];
