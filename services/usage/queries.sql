-- Queries over the usage reports, for Athena (workgroup pecofence-usage, region ap-northeast-1).
-- The table pecofence_usage.reports has one row per installation and day:
--   day       'YYYY-MM-DD', the sender's local date (the partition)
--   id        the installation's random number
--   app, edition, windows, language
--   settings, state, health   JSON texts: json_extract_scalar(settings, '$.theme')
--   events    in the order they happened. An event e is [second of the day, name, key]:
--             e[2] is the name, element_at(e, 3) the key (NULL when the event has none)
-- Reports are deleted after 180 days, so nothing here reaches further back.


-- 1. Installations per day.
SELECT day, count(*) AS installations
FROM pecofence_usage.reports
GROUP BY day
ORDER BY day DESC;


-- 2. How often each event occurred in the last 7 days, and in how many installations.
SELECT e[2] AS name,
       element_at(e, 3) AS key,
       count(*) AS times,
       count(DISTINCT id) AS installations
FROM pecofence_usage.reports
CROSS JOIN UNNEST(events) AS t (e)
WHERE day >= date_format(current_date - interval '7' day, '%Y-%m-%d')
GROUP BY 1, 2
ORDER BY times DESC;


-- 3. What most often follows a given event (here: hotkey), within the same installation and day.
WITH ordered AS (
  SELECT e[2] AS name,
         lead(e[2]) OVER (PARTITION BY day, id ORDER BY n) AS next
  FROM pecofence_usage.reports
  CROSS JOIN UNNEST(events) WITH ORDINALITY AS t (e, n)
  WHERE day >= date_format(current_date - interval '30' day, '%Y-%m-%d')
)
SELECT coalesce(next, '(nothing more that day)') AS next, count(*) AS times
FROM ordered
WHERE name = 'hotkey'
GROUP BY 1
ORDER BY times DESC
LIMIT 20;


-- 4. Of the installations first seen in one week, how many were seen again in the next.
--    The oldest week counts every installation that is older than the kept 180 days as new,
--    and the newest week has no next week yet: leave both out when reading the result.
WITH weeks AS (
  SELECT DISTINCT id, date_trunc('week', date(day)) AS week
  FROM pecofence_usage.reports
),
first_weeks AS (
  SELECT id, min(week) AS week
  FROM weeks
  GROUP BY id
)
SELECT f.week AS first_week,
       count(*) AS new_installations,
       count(w.id) AS seen_next_week,
       round(100.0 * count(w.id) / count(*), 1) AS percent
FROM first_weeks f
LEFT JOIN weeks w ON w.id = f.id AND w.week = f.week + interval '7' day
GROUP BY f.week
ORDER BY f.week;


-- 5. Versions and editions in use yesterday.
SELECT app, edition, count(*) AS installations
FROM pecofence_usage.reports
WHERE day = date_format(current_date - interval '1' day, '%Y-%m-%d')
GROUP BY app, edition
ORDER BY installations DESC;


-- 6. How a setting is set (here: theme), over the installations seen in the last 7 days.
--    Each installation counts once, with its latest report.
SELECT theme, count(*) AS installations
FROM (
  SELECT max_by(json_extract_scalar(settings, '$.theme'), day) AS theme
  FROM pecofence_usage.reports
  WHERE day >= date_format(current_date - interval '7' day, '%Y-%m-%d')
  GROUP BY id
)
GROUP BY theme
ORDER BY installations DESC;
