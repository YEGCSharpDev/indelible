-- Drop AI and Mila tables
DROP TABLE IF EXISTS public.ai_outputs CASCADE;
DROP TABLE IF EXISTS public.ai_runs CASCADE;
DROP TABLE IF EXISTS public.ai_prompt_presets CASCADE;
DROP TABLE IF EXISTS public.content_vectors CASCADE;
DROP TABLE IF EXISTS public.mila_messages CASCADE;
DROP TABLE IF EXISTS public.mila_sessions CASCADE;
DROP TABLE IF EXISTS public.mila_config CASCADE;

-- Drop entity extraction tables
DROP TABLE IF EXISTS public.entity_mentions CASCADE;
DROP TABLE IF EXISTS public.entity_aliases CASCADE;
DROP TABLE IF EXISTS public.entities CASCADE;

-- Drop TTS tables
DROP TABLE IF EXISTS public.tts_audio_assets CASCADE;
DROP TABLE IF EXISTS public.tts_chunks CASCADE;
DROP TABLE IF EXISTS public.tts_element_timings CASCADE;
DROP TABLE IF EXISTS public.tts_session_chunks CASCADE;
DROP TABLE IF EXISTS public.tts_sessions CASCADE;
DROP TABLE IF EXISTS public.tts_voice_personas CASCADE;

-- Remove Mila/AI preferences from user_preferences table
ALTER TABLE public.user_preferences 
  DROP COLUMN IF EXISTS ai_mila_enabled,
  DROP COLUMN IF EXISTS ai_custom_prompt,
  DROP COLUMN IF EXISTS ai_auto_processing;
