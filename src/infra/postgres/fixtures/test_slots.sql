INSERT INTO users (id, username, full_name_lat, full_name_cyr, citizenship, arrival_date)
VALUES
    (1, 'username1', 'Ivanov', 'Иванов', 'Tajikistan', '2025-07-10'),
    (2, 'username2', 'Sidorov', 'Сидоров', 'Armenia', '2025-07-01'),
    (3, 'username3', 'Petrov', 'Петров', 'Ukraine', '2025-07-01')
ON CONFLICT (id) DO NOTHING;

INSERT INTO reservations (slot_start, user_id)
VALUES
    (TIMESTAMP '2025-07-14 9:00', 1),
    (TIMESTAMP '2025-07-14 9:00', 2),
    (TIMESTAMP '2025-07-14 9:00', 3),
    (TIMESTAMP '2025-07-14 9:20', 1),
    (TIMESTAMP '2025-07-14 9:20', 2)
ON CONFLICT (slot_start, user_id) DO NOTHING;
