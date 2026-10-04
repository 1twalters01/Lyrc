CREATE TABLE lyrics_format (
    id INTEGER PRIMARY KEY,
    format TEXT NOT NULL UNIQUE
);

INSERT INTO lyrics_format (id, format) VALUES
    (1, 'text'),
    (2, 'lrc'),
    (3, 'elrc');



CREATE TABLE source (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL UNIQUE
);

INSERT INTO source (id, source) VALUES
    (1, 'musicbrainz'),
    (2, 'liblrc');



CREATE TABLE translation (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO translation (id, name) VALUES
    (1, 'original'),
    (2, 'translated');



CREATE TABLE language (
    id INTEGER PRIMARY KEY,
    code_3 TEXT NOT NULL UNIQUE
        CHECK (length(code_3) = 3)
);

INSERT INTO language (id, code) VALUES
    (1, 'epo'),
    (2, 'eng'),
    (3, 'rus'),
    (4, 'cmn'),
    (5, 'spa'),
    (6, 'por'),
    (7, 'ita'),
    (8, 'ben'),
    (9, 'fra'),
    (10, 'deu'),
    (11, 'ukr'),
    (12, 'kat'),
    (13, 'ara'),
    (14, 'hin'),
    (15, 'jpn'),
    (16, 'heb'),
    (17, 'yid'),
    (18, 'pol'),
    (19, 'amh'),
    (20, 'jav'),
    (21, 'kor'),
    (22, 'nob'),
    (23, 'dan'),
    (24, 'swe'),
    (25, 'fin'),
    (26, 'tur'),
    (27, 'nld'),
    (28, 'hun'),
    (29, 'ces'),
    (30, 'ell'),
    (31, 'bul'),
    (32, 'bel'),
    (33, 'mar'),
    (34, 'kan'),
    (35, 'ron'),
    (36, 'slv'),
    (37, 'hrv'),
    (38, 'srp'),
    (39, 'mkd'),
    (40, 'lit'),
    (41, 'lav'),
    (42, 'est'),
    (43, 'tam'),
    (44, 'vie'),
    (45, 'urd'),
    (46, 'tha'),
    (47, 'guj'),
    (48, 'uzb'),
    (49, 'pan'),
    (50, 'aze'),
    (51, 'ind'),
    (52, 'tel'),
    (53, 'pes'),
    (54, 'mal'),
    (55, 'ori'),
    (56, 'mya'),
    (57, 'nep'),
    (58, 'sin'),
    (59, 'khm'),
    (60, 'tuk'),
    (61, 'aka'),
    (62, 'zul'),
    (63, 'sna'),
    (64, 'afr'),
    (65, 'lat'),
    (66, 'slk'),
    (67, 'cat'),
    (68, 'tgl'),
    (69, 'hye'),
    (70, 'cym');
