INSERT INTO users (id, username, full_name_lat, full_name_cyr, citizenship, arrival_date)
VALUES
    (1, 'username1', 'Ivanov', 'Иванов', 'Tajikistan', '2025-07-10'),
    (2, 'username2', 'Sidorov', 'Сидоров', 'Armenia', '2025-07-01'),
    (3, 'username3', 'Petrov', 'Петров', 'Ukraine', '2025-07-01')
ON CONFLICT (id) DO NOTHING;

INSERT INTO reservations (slot_start, service, user_id)
VALUES
    (TIMESTAMP '2025-07-14 9:00', 'initial_registration', 1),
    (TIMESTAMP '2025-07-14 9:00', 'visa', 2),
    (TIMESTAMP '2025-07-14 9:00', 'all', 3),
    (TIMESTAMP '2025-07-14 9:20', 'all', 1),
    (TIMESTAMP '2025-07-14 9:20', 'renewal_of_visa', 2)
ON CONFLICT (slot_start, user_id) DO NOTHING;
