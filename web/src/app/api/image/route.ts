import { NextRequest, NextResponse } from 'next/server';
import { createClient } from '@/context/Supabase/server';

export async function POST(request: NextRequest) {
  const PREVIEW_IMAGE_BUCKET = 'og-preview-images';

  try {
    // Get the form data from the request
    const formData = await request.formData();
    const file = formData.get('image') as File;
    const threadId = formData.get('threadId') as string;

    if (!file) {
      return NextResponse.json({ error: 'No image provided' }, { status: 400 });
    }

    if (!threadId) {
      return NextResponse.json({ error: 'No threadId provided' }, { status: 400 });
    }

    // Create Supabase client
    const supabase = await createClient();

    // Upload to Supabase storage
    const { data, error } = await supabase.storage
      .from(PREVIEW_IMAGE_BUCKET)
      .upload(`${threadId}`, file, {
        cacheControl: '3600',
        upsert: true
      });

    if (error) {
      return NextResponse.json({ error: error.message }, { status: 500 });
    }

    // Get public URL
    const {
      data: { publicUrl }
    } = supabase.storage.from(PREVIEW_IMAGE_BUCKET).getPublicUrl(data.path);

    return NextResponse.json({ url: publicUrl });
  } catch (error) {
    console.error('error', error);
    return NextResponse.json('Internal server error - :(', { status: 500 });
  }
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const threadId = searchParams.get('threadId');
  return NextResponse.json({ message: 'Hello, world!' });
}
